cmake_minimum_required(VERSION 3.0)
project(LibMSRTools C)
add library(msrr STATIC msrr.c)
install(TARGETS msrr DESTINATION .)

