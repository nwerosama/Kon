#!/bin/bash

export $(grep -v '^#' .env | xargs) && cargo run kon_dev
