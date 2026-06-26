#!/bin/sh
exec musl-gcc -D'__GNUC_PREREQ(maj,min)=0' "$@"
