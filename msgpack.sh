#!/bin/bash
#
# Json to MessagePack encoder/decoder.
#
# Dependencies:
# - msgpack-tools: https://github.com/ludocode/msgpack-tools
# - xxd: standard unix tool for hex/bin conversions

function get_input() {
    if [ -z $1 ]; then
        input=$(</dev/stdin)
    else
        input=$@
    fi
}

function encode() {
    get_input $@
    echo $input | json2msgpack | xxd -p -c 256
}


function decode() {
    get_input $@
    echo $input | xxd -r -p | msgpack2json -d -b -c
}

function help() {
    echo "Usage: ./msgpack [-d|-e]"
    echo "  -e : encode json to message packed hex string"
    echo "  -d : decode message packed hex string to json"
}

while getopts "edh" arg; do
    case "${arg}" in
    e)
        encode $2
        run=1
        ;;
    d)
        decode $2
        run=1
        ;;
    h)
        help
        exit 0
        ;;
    ?)
        echo "Invalid option: -${OPTARG}"
        exit 2
        ;;
    esac
done

if [ -z $run ]; then
    help
fi
