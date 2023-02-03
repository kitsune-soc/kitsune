#!/usr/bin/env bash

sea-orm-cli generate entity --date-time-crate chrono --with-copy-enums -o kitsune-db/src/entity
