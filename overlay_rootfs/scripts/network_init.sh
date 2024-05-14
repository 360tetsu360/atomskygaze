#!/bin/sh

if [ "$1" = "stop" -o "$1" = "restart" ] ; then
  ifconfig | grep eth0 && ifconfig eth0 down
  killall -SIGKILL wpa_supplicant
  killall -SIGKILL udhcpc
  killall -SIGKILL hostapd
  killall -SIGKILL dnsmasq
  [ "$1" = "stop" ] && exit 0
fi

devmem 0x10011110 32 0x6e094800
devmem 0x10011138 32 0x300
devmem 0x10011134 32 0x300

ifconfig lo up

[ -x /media/mmc/network_init.sh ] && /media/mmc/network_init.sh start && exit

# USB-Ether
if lsusb | grep 'Device 002:' ; then
  for i in r8152 asix ax88179_178a cdc_ether ; do
    modprobe $i
    sleep 0.5
    if ip link | grep eth0 > /dev/null ; then
      ifconfig eth0 up
      udhcpc -i eth0 -x hostname:atomcam -p /var/run/udhcpc.pid -b
      for count in `seq 20` ; do
        ifconfig eth0 | grep 'inet addr' > /dev/null && exit 0
        sleep 0.5
      done
    fi
    rmmod $i
  done
fi

# WiFi
VENDERID="0x024c"
if [ -f /atom/system/driver/mmc_detect_test.ko ]; then
  insmod /atom/system/driver/mmc_detect_test.ko
  while [ ! -f /sys/bus/mmc/devices/mmc1\:0001/mmc1\:0001\:1/vendor ]; do
    sleep 0.1
  done
  VENDERID=`cat /sys/bus/mmc/devices/mmc1\:0001/mmc1\:0001\:1/vendor`
fi
if [ "0x024c" = "$VENDERID" ]; then
  insmod /atom/system/driver/rtl8189ftv.ko
elif [ "0x007a" = "$VENDERID" ]; then
  [ -f /atom/usr/share/atbm603x_conf/atbm_txpwer_dcxo_cfg.txt ] && cp /atom/usr/share/atbm603x_conf/atbm_txpwer_dcxo_cfg.txt /tmp
  [ -f /atom/usr/share/atbm603x_conf/set_rate_power.txt ] && cp /atom/usr/share/atbm603x_conf/set_rate_power.txt /tmp
  insmod /lib/modules/atbm603x_wifi_sdio.ko
  count=0
  while [ "`cat /sys/module/atbm603x_wifi_sdio/initstate 2>&1`" != "live" ] ; do
    sleep 0.5
    let count++
    [ 20 -le $count ] && break
  done
  echo "LOG_ERR=OFF LOG_SCAN=OFF" > /sys/module/atbm603x_wifi_sdio/atbmfs/atbm_printk_mask 2> /dev/null
elif [ "0x5653" = "$VENDERID" ]; then
  insmod /atom/system/driver/ssv6x5x.ko stacfgpath=/atom/system/driver/ssv6x5x-wifi.cfg
elif [ "0x424c" = "$VENDERID" ]; then
    insmod /atom/system/driver/bl_fdrv.ko
fi

# AP_MODE or CLIENT_MODE
CONFIG_FLAG=""

config_file="/media/mmc/atom_config.toml"
get_value() {
  grep "^$1" "$config_file" | awk -F ' = ' '{print $2}' | tr -d '"'
}

if [ -f "$config_file" ]; then
  echo "'$config_file' found!"


  ap_mode=$(get_value "ap_mode")

  if [[ "$ap_mode" == "true" ]]; then
    echo "Enable AP_MODE"
    CONFIG_FLAG="AP_MODE"
  elif [[ "$ap_mode" == "false" ]]; then
    echo "Disable AP_MODE"
    CONFIG_FLAG="CLIENT_MODE"
  else 
    echo "Can't read config file."
  fi
fi


if [[ -z "$CONFIG_FLAG" || "$CONFIG_FLAG" == "CLIENT_MODE" ]]; then
  echo "Configure CLIENT mode"
  if [ -f /media/mmc/wpa_supplicant.conf ] ; then
    CONFIG_FLAG="CLIENT_MODE"
    cat /media/mmc/wpa_supplicant.conf > /configs/etc/wpa_supplicant.conf
  else
    USER_CONFIG=/atom/configs/.user_config
    SSID=$(awk -F "=" '/\[NET\]/ { f = 1; } /ssid=/ {if(!f) next; gsub(/\/$/, "", $2); print $2}' $USER_CONFIG)
    PSK=$(awk -F "=" '/\[NET\]/ { f = 1; } /password=/ {if(!f) next; gsub(/\/$/, "", $2); print $2}' $USER_CONFIG)
    if [ -n "$SSID" ] && [ -n "$PSK" ]; then
      CONFIG_FLAG="CLIENT_MODE"
      cat > /configs/etc/wpa_supplicant.conf << EOF
ctrl_interface=/var/run/wpa_supplicant
update_config=1
network={
  ssid="$SSID"
  psk="$PSK"
  scan_ssid=1
}
EOF

    fi
  fi
fi

if [ "$CONFIG_FLAG" == "CLIENT_MODE" ]; then
  echo "Starting in CLIENT mode"
  count=0
  while ! ip link | grep wlan0 > /dev/null ; do
    sleep 0.5
    let count++
    [ 20 -le $count ] && break
  done

  HWADDR=$(awk -F "=" '/(CONFIG_INFO|NETRELATED_MAC)=/ { print substr($2,1,2) ":" substr($2,3,2) ":" substr($2,5,2) ":" substr($2,7,2) ":" substr($2,9,2) ":" substr($2,11,2); exit;}' /atom/configs/.product_config)
  ifconfig wlan0 hw ether $HWADDR up
  wpa_supplicant -f /tmp/log/wpa_supplicant.log -D nl80211 -i wlan0 -c /configs/etc/wpa_supplicant.conf -B
  udhcpc -i wlan0 -x hostname:atomskygaze -p /var/run/udhcpc.pid -b &

  count=0
  while ! ifconfig wlan0 | grep 'inet addr' > /dev/null
  do
    sleep 0.5
    let count++
    [ 20 -le $count ] && break
  done
elif [ "$CONFIG_FLAG" == "AP_MODE" ]; then
  echo "Starting in AP mode"
  HWADDR=$(awk -F "=" '/(CONFIG_INFO|NETRELATED_MAC)=/ { print substr($2,1,2) ":" substr($2,3,2) ":" substr($2,5,2) ":" substr($2,7,2) ":" substr($2,9,2) ":" substr($2,11,2); exit;}' /atom/configs/.product_config)
  ifconfig wlan0 hw ether $HWADDR up
  HOSTNAME=$(get_value "hostname")
  SSID=$(get_value "ssid")
  PSK=$(get_value "psk")

  cat > /configs/etc/hostapd.conf << EOF
interface=wlan0
logger_syslog=-1
logger_syslog_level=2
logger_stdout=-1
logger_stdout_level=2
ctrl_interface=/var/run/hostapd
ctrl_interface_group=0
ssid=$SSID
hw_mode=g
channel=3
beacon_int=100
dtim_period=2
max_num_sta=255
rts_threshold=-1
fragm_threshold=-1
macaddr_acl=0
auth_algs=3
ignore_broadcast_ssid=0
wmm_enabled=1
wmm_ac_bk_cwmin=4
wmm_ac_bk_cwmax=10
wmm_ac_bk_aifs=7
wmm_ac_bk_txop_limit=0
wmm_ac_bk_acm=0
wmm_ac_be_aifs=3
wmm_ac_be_cwmin=4
wmm_ac_be_cwmax=10
wmm_ac_be_txop_limit=0
wmm_ac_be_acm=0
wmm_ac_vi_aifs=2
wmm_ac_vi_cwmin=3
wmm_ac_vi_cwmax=4
wmm_ac_vi_txop_limit=94
wmm_ac_vi_acm=0
wmm_ac_vo_aifs=2
wmm_ac_vo_cwmin=2
wmm_ac_vo_cwmax=3
wmm_ac_vo_txop_limit=47
wmm_ac_vo_acm=0
eapol_key_index_workaround=0
own_ip_addr=127.0.0.1
wpa=2
wpa_passphrase=$PSK
wpa_key_mgmt=WPA-PSK
rsn_pairwise=CCMP
EOF

  hostapd -B /configs/etc/hostapd.conf
	[ $? = 0 ] && echo "OK" || echo "FAIL"

  cat > /configs/etc/dnsmasq.conf << EOF
interface=wlan0
dhcp-range=192.168.2.2,192.168.2.10,255.255.255.0,24h
domain=local
address=/$HOSTNAME.local/192.168.2.1
EOF

  ifconfig wlan0 192.168.2.1 netmask 255.255.255.0 up

  dnsmasq -C /configs/etc/dnsmasq.conf
else
  echo "Unknown mode"
fi
