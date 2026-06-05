/*
 
 
SOURCE: 
   nvim /opt/esp-idf/examples/wifi/softap_sta/main/softap_sta.c
 
 
  */

#include <string.h>
#include "esp_err.h"
#include "esp_wifi_default.h"
#include "esp_wifi_types_generic.h"
#include "esp_wifi_types_native.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_mac.h"
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "nvs.h"
#include "nvs_flash.h"

// AP Config <- this important
#define ESP_WIFI_AP_SSID        "ESPresso"
#define ESP_WIFI_AP_PASSWORD    "12345678"
#define ESP_WIFI_AP_CHANNEL         1
#define ESP_WIFI_AP_MAX_CONNECTION  4 
#define ESP_WIFI_AUTH_MODE      WIFI_AUTH_WPA2_PSK


static const char *TAG_AP = "WiFi SoftAP";

// WIFI HANDLER
static void wifi_event_handler(
        void *arg, 
        esp_event_base_t event_base,
        int32_t event_id, void *event_data) 
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_AP_STACONNECTED) {
        wifi_event_ap_staconnected_t *event = (wifi_event_ap_staconnected_t *) event_data;
        ESP_LOGI(TAG_AP, "Station "MACSTR" joined, AID=%d",
                MAC2STR(event->mac), event->aid);
    }  else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_AP_STADISCONNECTED) {
        wifi_event_ap_stadisconnected_t *event = (wifi_event_ap_stadisconnected_t *) event_data;
        ESP_LOGI(TAG_AP, "Station "MACSTR" left, AID=%d, reason:%d",
                 MAC2STR(event->mac), event->aid, event->reason);
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
    
    printf("ESPresso booting...\n");
}
