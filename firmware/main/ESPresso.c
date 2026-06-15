/*
 
 
SOURCE: 
    SOFT AP WIFI
    nvim ~/esp-idf/examples/wifi/softap_sta/main/softap_sta.c

    HTTP Server
    nvim ~/esp-idf/examples/protocols/http_server/simple/main/main.c

    WEBSOCKET
    nvim ~/esp-idf/examples/protocols/http_server/ws_echo_server/main/ws_echo_server.c
    */

#include <string.h>
#include <esp_http_server.h>
#include "esp_err.h"
#include "esp_wifi_default.h"
#include "esp_wifi_types_generic.h"
#include "freertos/task.h"
#include "esp_mac.h"
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "nvs.h"
#include "nvs_flash.h"
#include "cJSON.h"

// AP Config <- this important
#define ESP_WIFI_AP_SSID        "ESPresso"
#define ESP_WIFI_AP_PASSWORD    "12345678"
#define ESP_WIFI_AP_CHANNEL         1
#define ESP_WIFI_AP_MAX_CONNECTION  4 
#define ESP_WIFI_AUTH_MODE      WIFI_AUTH_WPA2_PSK

// Websocket Config
#define MAX_CLIENTS 4 // same as ESP_WIFI_AP_MAX_CONNECTION

// JSON struct
#define MAX_NAME_LEN 32
#define MAX_ROLE_LEN 64
#define MAX_BIO_LEN 128

typedef struct {
    char name[MAX_NAME_LEN];
    char role[MAX_ROLE_LEN];
    char bio[MAX_BIO_LEN];
    int fd;
    bool active;
} Profile;

static httpd_handle_t server_handle = NULL;
static const char *TAG_AP = "WiFi SoftAP";

// struct holding the server handle
// internal socket fd and to use request send 
struct async_resp_arg {
    httpd_handle_t hd;
    int fd;
};

// WIFI HANDLER
static void wifi_event_handler(
        void *arg, 
        esp_event_base_t event_base,
        int32_t event_id, void *event_data) 
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_AP_STACONNECTED) {
        wifi_event_ap_staconnected_t *event = (wifi_event_ap_staconnected_t *) event_data;
        ESP_LOGI(TAG_AP, "Station "MACSTR" joined, AID=%d",
                MAC2STR(event -> mac), event -> aid);
    }  else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_AP_STADISCONNECTED) {
        wifi_event_ap_stadisconnected_t *event = (wifi_event_ap_stadisconnected_t *) event_data;
        ESP_LOGI(TAG_AP, "Station "MACSTR" left, AID=%d, reason:%d",
                 MAC2STR(event -> mac), event -> aid, event -> reason);
    } 

}

// WIFI AP INITIALIZE
esp_netif_t *wifi_init_softap(void) {
    esp_netif_t *esp_netif_ap = esp_netif_create_default_wifi_ap();

    // AP CONFIG
    wifi_config_t wifi_ap_config = {
        .ap = {
            .ssid = ESP_WIFI_AP_SSID,
            .ssid_len = strlen(ESP_WIFI_AP_SSID),
            .channel = ESP_WIFI_AP_CHANNEL,
            .password = ESP_WIFI_AP_PASSWORD,
            .max_connection = ESP_WIFI_AP_MAX_CONNECTION,
            .authmode = ESP_WIFI_AUTH_MODE,
            .pmf_cfg = {
                .required = false,
            },
        },
    };

    if (strlen(ESP_WIFI_AP_PASSWORD) == 0) {
        wifi_ap_config.ap.authmode = WIFI_AUTH_OPEN;
    }

    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_AP, &wifi_ap_config));

    ESP_LOGI(TAG_AP, "wifi_init_softap finished. SSID:%s password:%s channel:%d",
            ESP_WIFI_AP_SSID, ESP_WIFI_AP_PASSWORD, ESP_WIFI_AP_CHANNEL);

    return esp_netif_ap;
}


// BROADCAST FUNCTION
static void broadcast(httpd_handle_t hd, httpd_ws_frame_t *pkt) {
    size_t clients  = MAX_CLIENTS;
    int client_fds[MAX_CLIENTS];

    if (httpd_get_client_list(hd, &clients, client_fds) == ESP_OK) {
        for (int i = 0; i < clients; i++) {
            if (httpd_ws_get_fd_info(hd, client_fds[i]) ==  HTTPD_WS_CLIENT_WEBSOCKET) {
                httpd_ws_send_frame_async(hd, client_fds[i], pkt);
            }
        }
    }
}

// Websocket Handler
static esp_err_t ws_handler(httpd_req_t *req) {
    httpd_ws_frame_t ws_pkt;
    uint8_t *buf = NULL;
    memset(&ws_pkt, 0, sizeof(httpd_ws_frame_t));
    ws_pkt.type = HTTPD_WS_TYPE_TEXT;

    if (req -> method == HTTP_GET) {
        ESP_LOGI(TAG_AP, "Client Connected");
        return ESP_OK;
    }

    esp_err_t ret = httpd_ws_recv_frame(req, &ws_pkt, 0);
    if (ret != ESP_OK) {
        ESP_LOGE(TAG_AP, "httpd_ws_recv_frame failed to get frame len with %d", ret);
        return ret;
    }
    ESP_LOGI(TAG_AP, "frame len is %d", ws_pkt.len);
    if (ws_pkt.len) {
        buf = calloc(1, ws_pkt.len + 1);
        if (buf == NULL) {
            ESP_LOGE(TAG_AP, "Failed to calloc memory for buf");
            return ESP_ERR_NO_MEM;
        }
        ws_pkt.payload = buf;
        ret = httpd_ws_recv_frame(req, &ws_pkt, ws_pkt.len);
        if (ret != ESP_OK) {
            ESP_LOGE(TAG_AP, "httpd_ws_recv_frame failed with %d", ret);
            free(buf);
            return ret;
        }
        ESP_LOGI(TAG_AP, "Got packet with message: %s", ws_pkt.payload);
    }

    ESP_LOGI(TAG_AP, "Packet type: %d", ws_pkt.type);
    if (ws_pkt.type == HTTPD_WS_TYPE_TEXT 
            && ws_pkt.payload != NULL) {
        broadcast(req -> handle, &ws_pkt);
    }

    free(buf);
    return ESP_OK;
}

// WS URI
static const httpd_uri_t ws = {
    .uri = "/ws",
    .method = HTTP_GET,
    .handler = ws_handler,
    .is_websocket = true,
};

// WEBSOCKET ASYNC SEND
static void ws_async_send(void *arg) {
    static const char *data = "Async data";
    struct async_resp_arg *resp_arg = arg;
    httpd_handle_t hd = resp_arg -> hd;
    int fd = resp_arg -> fd;
    httpd_ws_frame_t ws_pkt;
    memset(&ws_pkt, 0, sizeof(httpd_ws_frame_t));
    ws_pkt.payload = (uint8_t*)data;
    ws_pkt.len = strlen(data);
    ws_pkt.type = HTTPD_WS_TYPE_TEXT;

    httpd_ws_send_frame_async(hd, fd, &ws_pkt);
    free(resp_arg);
}

// TRIGGER ASYNC SEND
static esp_err_t trigger_async_send(httpd_handle_t handle, httpd_req_t *req) {
    struct async_resp_arg *resp_arg = malloc(sizeof(struct async_resp_arg));
    if (resp_arg == NULL) return ESP_ERR_NO_MEM;
    
    resp_arg -> hd = req -> handle;
    resp_arg -> fd = httpd_req_to_sockfd(req);
    esp_err_t ret = httpd_queue_work(handle, ws_async_send, resp_arg);
    if (ret != ESP_OK) free(resp_arg);

    return ret;
}

static void ping_task(void *arg) {
    while (1) {
        vTaskDelay(pdMS_TO_TICKS(10000));
        if (server_handle == NULL) continue;
        
        size_t clients = MAX_CLIENTS;
        int client_fds[MAX_CLIENTS];

        if (httpd_get_client_list(server_handle, &clients, client_fds) == ESP_OK) {
            for (int i = 0; i < clients; i++) {
                if (httpd_ws_get_fd_info(server_handle, client_fds[i]) == HTTPD_WS_CLIENT_WEBSOCKET) {
                    httpd_ws_frame_t ping = {0};
                    ping.type = HTTPD_WS_TYPE_PING;
                    httpd_ws_send_frame_async(server_handle, client_fds[i], &ping);
                    ESP_LOGI(TAG_AP, "Ping sent to fd %d", client_fds[i]);
                }
            }
        }
    }
}

// HTTP Handler 
static esp_err_t main_get_handler(httpd_req_t *req) {
    const char* html = "<h1>ESPresso TEST</h1>";
    httpd_resp_send(req, html, HTTPD_RESP_USE_STRLEN);
    return ESP_OK;
}

// GET 
static const httpd_uri_t main = {
    .uri = "/",
    .method = HTTP_GET,
    .handler = main_get_handler,
};

static httpd_handle_t start_webserver(void) {
    httpd_handle_t server = NULL;
    httpd_config_t config = HTTPD_DEFAULT_CONFIG();

    if (httpd_start(&server, &config) == ESP_OK) {
        // SET URI HANDLERS
        ESP_LOGI(TAG_AP, "Server Started");
        httpd_register_uri_handler(server, &main);
        httpd_register_uri_handler(server, &ws);
        return server;
    }

    ESP_LOGI(TAG_AP, "Server failed to start.");
    return NULL;
}

void app_main(void)
{

    // Initialize NVS
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);

    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    // Event Handler register idk
    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT,
                    ESP_EVENT_ANY_ID,
                    &wifi_event_handler,
                    NULL,
                    NULL));
 

    // Initializes WiFi
    // SOURCE: https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv413esp_wifi_initPK18wifi_init_config_t
    wifi_init_config_t wifi_config = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&wifi_config));
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_AP));


    wifi_init_softap();

    ESP_ERROR_CHECK(esp_wifi_start());
    
    server_handle = start_webserver();
    xTaskCreate(ping_task, "ping_task", 4096, NULL, 5, NULL);

    // while running
    while (1) {
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
    
    printf("ESPresso booting...\n");
}
