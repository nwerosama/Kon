#!/bin/bash

export $(cat .env | xargs) && cargo run
