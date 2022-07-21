#!/bin/sh

SCRIPT_PATH=$(dirname "$0")
SERVICE="acs.service"
SERVICE_PATH=$SCRIPT_PATH/../$SERVICE
TMP_SERVICE=`mktemp`
LONG_ARGS="help,acs-path:,user:"
ACS_PATH=""
SERVICE_ARGS="run --no-animation --quiet"

usage() {
  cat <<EOF
Usage

$(basename $0) [[--acs-path ACS_INSTALLED_PATH] | [ --user USER ]]

--acs-path: specify installed path for the acs binary
--user: enforce user. Service will be installed with ExecStart relative
        to the user homedir (~/.cargo/bin/acs)

EOF
}

clean_up () {
  [ -f "$TMP_SERVICE" ] && rm $TMP_SERVICE
}

setup_service() {
  mv -f $TMP_SERVICE /etc/systemd/system/$SERVICE\
  && chmod 644 /etc/systemd/system/$SERVICE \
  && chown root:root /etc/systemd/system/$SERVICE \
  && systemctl daemon-reload
}

start_service() {
  systemctl restart acs
  systemctl enable acs
  systemctl status acs
}

# Main
# read the options
OPTS=$(getopt -o '' -a -l $LONG_ARGS --name "$0" -- "$@")

# On parsin failure
if [ $? != 0 ] ; then
  usage
  exit 1 ;
fi

eval set -- "$OPTS"

while true ; do
  case "$1" in
    --acs-path )
      ACS_PATH=$2
      shift 2
      ;;
    --user )
      ARG_USER=$2
      shift 2
      ;;
    --help )
      usage
      exit
      ;;
     -- )
      shift
      break
      ;;
  esac
done

if [ ! -z "$ARG_USER" ]; then
  HOME_DIR=$(getent passwd $ARG_USER | awk -F ':' '{ print $6 }')
  ACS_PATH=$HOME_DIR/.cargo/bin/acs
  sed -E "s@^ExecStart=.*@ExecStart=$ACS_PATH $SERVICE_ARGS@g" $SERVICE_PATH > $TMP_SERVICE
elif [ ! -z "$ACS_PATH" ]; then
  sed -E "s@^ExecStart=.*@ExecStart=$ACS_PATH $SERVICE_ARGS@g" $SERVICE_PATH > $TMP_SERVICE
else
  usage
  exit 1
fi

setup_service && start_service
clean_up

