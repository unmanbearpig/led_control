#!/bin/sh

set -e

make fade_test | grep '<' | grep -v '0000 0000 0000 0000 0000'
