#!/bin/sh

set -e

# source $IDF_PATH/bin/activate
cd esp8266_wifi_bridge/
make CONFIG_SDK_TOOLPREFIX=$CONFIG_SDK_TOOLPREFIX IDF_PATH=$IDF_PATH
