#!/bin/bash

# 設定ファイルの名前
config_file="./atomconf.toml"

# 設定ファイルが存在するか確認
if [ ! -f "$config_file" ]; then
  echo "Error: File '$config_file' not found!"
  exit 1
fi

# 設定データから特定の行を抽出し、値を得るための関数
get_value() {
    grep "^$1" "$config_file" | awk -F ' = ' '{print $2}' | tr -d '"'
}

# 設定データから特定の値を取得
hostname=$(get_value "hostname")
ap_mode=$(get_value "ap_mode")
ssid=$(get_value "ssid")
psk=$(get_value "psk")

# 結果を表示
echo "Hostname: $hostname"
echo "AP Mode: $ap_mode"
echo "SSID: $ssid"
echo "PSK: $psk"
