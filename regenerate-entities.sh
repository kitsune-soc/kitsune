#!/usr/bin/env bash

sea-orm-cli generate entity --date-time-crate time --with-copy-enums --with-serde both -o kitsune-db/src/entity
echo "==================================="
echo "Finished generating the entities. Make sure you replace all the enum fields with their custom enum definitions!"
