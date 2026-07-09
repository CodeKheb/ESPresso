idf_component_register(SRCS "ESPresso.c"
                    INCLUDE_DIRS "."
                    REQUIRES esp_wifi nvs_flash esp_netif esp_http_server 
                    PRIV_REQUIRES espressif__cjson esp_timer)
