#!/bin/bash

export $(grep -v '^#' .env.bot | xargs)
clear && cargo fmt && cargo run kon_dev
unset $(grep -v '^#' .env.bot | cut -d= -f1)
