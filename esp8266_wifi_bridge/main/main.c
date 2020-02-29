
///// https://gist.github.com/Bresenham/f5c016361e21b9a3711ef5e7565f7874
#include <stdio.h>
#include <string.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "esp_system.h"
#include "esp_spi_flash.h"
#include "esp_log.h"

#include "driver/gpio.h"
#include "driver/spi.h"

#define	LED_OUTPUT_PIN_SEL	( (1 << GPIO_NUM_2) | (1 << GPIO_NUM_2) )

#define SPI_CS				GPIO_NUM_15
#define SPI_CS_PIN_SEL		( (1 << SPI_CS) | (1 << SPI_CS) )

void send_spi(uint8_t *data, int length) {
	spi_trans_t trans;
	memset(&trans, 0, sizeof(trans));

	trans.bits.mosi = length * 8;
	trans.mosi = data;
	trans.addr = NULL;
	trans.cmd = NULL;

	spi_trans(HSPI_HOST, &trans);
}

uint8_t read_spi() {
	uint8_t resp[1];

	spi_trans_t trans;
	memset(&trans, 0, sizeof(trans));

	trans.bits.miso = 8;
	trans.miso = resp;
	trans.addr = NULL;
	trans.cmd = NULL;

	spi_trans(HSPI_HOST, &trans);

	return resp[0];
}

void ICACHE_FLASH_ATTR spi_master_write_slave_task(void *arg) {
	printf("SPI Write-Task started...\n");

  while(true) {
    uint16_t buf[5] = { 0x1324, 0xbaef, 0xfeed, 0xbeef, 0xfafa };

    uint16_t v = 0;

    for(; v < 0xFFFF; v++) {
      buf[1] = v;
      buf[2] = v;
      buf[3] = v;
      buf[4] = v;

      send_spi(buf, sizeof(buf));

      if (v % 100 == 0) {
        vTaskDelay(1);
      }
    }

    for(; v > 0; v--) {
      buf[1] = v;
      buf[2] = v;
      buf[3] = v;
      buf[4] = v;

      send_spi(buf, sizeof(buf));
      if (v % 100 == 0) {
        vTaskDelay(1);
      }
    }
	}
}

void setup_spi() {
	gpio_config_t io_conf;
	io_conf.mode = GPIO_MODE_OUTPUT;

  // don't need it?
	io_conf.pin_bit_mask = LED_OUTPUT_PIN_SEL;
	io_conf.pull_down_en = 0;
	io_conf.pull_up_en = 0;
	gpio_config(&io_conf);

	spi_config_t spi_config;
	spi_config.interface.val = SPI_DEFAULT_INTERFACE;
	spi_config.mode = SPI_MASTER_MODE;
	spi_config.clk_div = SPI_40MHz_DIV;
	spi_config.event_cb = NULL;
	spi_init(HSPI_HOST, &spi_config);
}

void print_chip_info() {
  /* Print chip information */
  esp_chip_info_t chip_info;
  esp_chip_info(&chip_info);

  printf("This is ESP8266 chip with %d CPU cores, WiFi, ",
         chip_info.cores);

  printf("silicon revision %d, ", chip_info.revision);

  printf("%dMB %s flash\n", spi_flash_get_chip_size() / (1024 * 1024),
         (chip_info.features & CHIP_FEATURE_EMB_FLASH) ? "embedded" : "external");

}

/* void ICACHE_FLASH_ATTR app_main() { */
/*   setup_spi(); */
/*   print_chip_info(); */

/* 	xTaskCreate(spi_master_write_slave_task, "spi_master_write_slave", 2048, NULL, 3, NULL); */
/* } */



/* WiFi station Example

   This example code is in the Public Domain (or CC0 licensed, at your option.)

   Unless required by applicable law or agreed to in writing, this
   software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
   CONDITIONS OF ANY KIND, either express or implied.
*/
#include <string.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/event_groups.h"
#include "esp_system.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_event.h"
#include "esp_wifi.h"
#include "nvs.h"
#include "nvs_flash.h"

#include "lwip/err.h"
#include "lwip/sys.h"

#include "../../common/secrets.h"

/* The examples use WiFi configuration that you can set via project configuration menu

   If you'd rather not, just change the below entries to strings with
   the config you want - ie #define EXAMPLE_WIFI_SSID "mywifissid"
*/

#define EXAMPLE_ESP_MAXIMUM_RETRY  CONFIG_ESP_MAXIMUM_RETRY

/* FreeRTOS event group to signal when we are connected*/
static EventGroupHandle_t s_wifi_event_group;

/* The event group allows multiple bits for each event, but we only care about two events:
 * - we are connected to the AP with an IP
 * - we failed to connect after the maximum amount of retries */
#define WIFI_CONNECTED_BIT BIT0
#define WIFI_FAIL_BIT      BIT1

static const char *TAG = "wifi station";

static int s_retry_num = 0;


#include "lwipopts.h"
#include "lwip/sockets.h"
#include "lwip/ip_addr.h"
#include "lwip/init.h"
#include "lwip/netif.h"
#include "lwip/igmp.h"
#include "lwip/udp.h"
#include "lwip/udp.h"

/* void ICACHE_FLASH_ATTR handle_udp_recv(void *arg, struct udp_pcb *pcb, struct pbuf *p,  ip_addr_t *addr, u16_t port) { */
/* 		int length = p->len; */
/* 		char * pusrdata = p->payload; */
/* 		os_printf("Received udp data: %s \r\n", pusrdata); */
/* 		pbuf_free(p); */
/* 	} */

/* 	void init_udp() { */
/* 		ip_addr_t ipSend; */
/* 		lwip_init(); */
/* 		struct udp_pcb * pUdpConnection = udp_new(); */
/* 		IP4_ADDR(&ipSend, 255, 255, 255, 255); */
/* 		// pUdpConnection->multicast_ip = ipSend; */
/* 		pUdpConnection->remote_ip = ipSend; */
/* 		pUdpConnection->remote_port = 8080; */
/* 		if(pUdpConnection == NULL) { */
/* 			os_printf("\nCould not create new udp socket... \n"); */
/* 		} */
/* 		// err = udp_bind(pUdpConnection, IP_ADDR_ANY, 8080); */
/*     udp_bind(pUdpConnection, IP_ADDR_ANY, 8080); */
/* 		udp_recv(pUdpConnection, handle_udp_recv, pUdpConnection); */
/* 	} */



static void event_handler(void* arg, esp_event_base_t event_base,
                                int32_t event_id, void* event_data)
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
        esp_wifi_connect();
    } else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {
        if (s_retry_num < EXAMPLE_ESP_MAXIMUM_RETRY) {
            esp_wifi_connect();
            s_retry_num++;
            ESP_LOGI(TAG, "retry to connect to the AP");
        } else {
            xEventGroupSetBits(s_wifi_event_group, WIFI_FAIL_BIT);
        }
        ESP_LOGI(TAG,"connect to the AP fail");
    } else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
        ESP_LOGI(TAG, "got ip:%s",
                 ip4addr_ntoa(&event->ip_info.ip));
        s_retry_num = 0;
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    }
}



void wifi_init_sta(void)
{
    s_wifi_event_group = xEventGroupCreate();

    tcpip_adapter_init();

    ESP_ERROR_CHECK(esp_event_loop_create_default());

    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));

    ESP_ERROR_CHECK(esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &event_handler, NULL));
    ESP_ERROR_CHECK(esp_event_handler_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &event_handler, NULL));

    wifi_config_t wifi_config = {
        .sta = {
            .ssid = WIFI_SSID,
            .password = WIFI_PASS
        },
    };
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA) );
    ESP_ERROR_CHECK(esp_wifi_set_config(ESP_IF_WIFI_STA, &wifi_config) );
    ESP_ERROR_CHECK(esp_wifi_start() );

    ESP_LOGI(TAG, "wifi_init_sta finished.");

    /* Waiting until either the connection is established (WIFI_CONNECTED_BIT) or connection failed for the maximum
     * number of re-tries (WIFI_FAIL_BIT). The bits are set by event_handler() (see above) */
    EventBits_t bits = xEventGroupWaitBits(s_wifi_event_group,
            WIFI_CONNECTED_BIT | WIFI_FAIL_BIT,
            pdFALSE,
            pdFALSE,
            portMAX_DELAY);

    /* xEventGroupWaitBits() returns the bits before the call returned, hence we can test which event actually
     * happened. */
    if (bits & WIFI_CONNECTED_BIT) {
        ESP_LOGI(TAG, "connected to ap SSID:%s password:%s",
                 WIFI_SSID, WIFI_PASS);
    } else if (bits & WIFI_FAIL_BIT) {
        ESP_LOGI(TAG, "Failed to connect to SSID:%s, password:%s",
                 WIFI_SSID, WIFI_PASS);
    } else {
        ESP_LOGE(TAG, "UNEXPECTED EVENT");
    }

    ESP_ERROR_CHECK(esp_event_handler_unregister(IP_EVENT, IP_EVENT_STA_GOT_IP, &event_handler));
    ESP_ERROR_CHECK(esp_event_handler_unregister(WIFI_EVENT, ESP_EVENT_ANY_ID, &event_handler));
    vEventGroupDelete(s_wifi_event_group);
}

#define PORT 8932

static void udp_server_task(void *pvParameters)
{
    char rx_buffer[128];
    char addr_str[128];
    int addr_family;
    int ip_protocol;

    while (1) {
        struct sockaddr_in destAddr;
        destAddr.sin_addr.s_addr = htonl(INADDR_ANY);
        destAddr.sin_family = AF_INET;
        destAddr.sin_port = htons(PORT);
        addr_family = AF_INET;
        ip_protocol = IPPROTO_IP;
        inet_ntoa_r(destAddr.sin_addr, addr_str, sizeof(addr_str) - 1);

        int sock = socket(addr_family, SOCK_DGRAM, ip_protocol);
        if (sock < 0) {
            ESP_LOGE(TAG, "Unable to create socket: errno %d", errno);
            break;
        }
        ESP_LOGI(TAG, "Socket created");

        int err = bind(sock, (struct sockaddr *)&destAddr, sizeof(destAddr));
        if (err < 0) {
            ESP_LOGE(TAG, "Socket unable to bind: errno %d", errno);
        }
        ESP_LOGI(TAG, "Socket binded");

        while (1) {
            /* ESP_LOGI(TAG, "Waiting for data"); */
            struct sockaddr_in sourceAddr;
            socklen_t socklen = sizeof(sourceAddr);
            int len = recvfrom(sock, rx_buffer, sizeof(rx_buffer) - 1, 0, (struct sockaddr *)&sourceAddr, &socklen);

            // Error occured during receiving
            if (len < 0) {
                ESP_LOGE(TAG, "recvfrom failed: errno %d", errno);
                break;
            }
            // Data received
            else {
                // Get the sender's ip address as string
                inet_ntoa_r(((struct sockaddr_in *)&sourceAddr)->sin_addr.s_addr, addr_str, sizeof(addr_str) - 1);

                //rx_buffer[len] = 0; // Null-terminate whatever we received and treat like a string...
                /* ESP_LOGI(TAG, "Received %d bytes from %s:", len, addr_str); */
                /* ESP_LOGI(TAG, "%s", rx_buffer); */

                send_spi((uint8_t *)rx_buffer, len);

                int err = sendto(sock, rx_buffer, len, 0, (struct sockaddr *)&sourceAddr, sizeof(sourceAddr));
                if (err < 0) {
                    ESP_LOGE(TAG, "Error occured during sending: errno %d", errno);
                    break;
                }
            }
        }

        if (sock != -1) {
            ESP_LOGE(TAG, "Shutting down socket and restarting...");
            shutdown(sock, 0);
            close(sock);
        }
    }
    vTaskDelete(NULL);
}


void app_main()
{
  printf("ssid=%s\n", WIFI_SSID);
  printf("pass=%s\n", WIFI_PASS);

    setup_spi();
    /* xTaskCreate(spi_master_write_slave_task, "spi_master_write_slave", 2048, NULL, 3, NULL); */

    ESP_ERROR_CHECK(nvs_flash_init());

    ESP_LOGI(TAG, "ESP_WIFI_MODE_STA");
    wifi_init_sta();

    xTaskCreate(udp_server_task, "udp_server", 4096, NULL, 5, NULL);
}
