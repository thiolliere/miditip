#!/bin/sh


pkill vkeybd > /dev/null 2>&1
pkill timidity > /dev/null 2>&1

timidity -f -B 1.1 -iA > /dev/null 2>&1 &
vkeybd > /dev/null 2>&1 

pkill vkeybd > /dev/null 2>&1
pkill timidity > /dev/null 2>&1

