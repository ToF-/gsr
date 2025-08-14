#!/bin/bash
sqlite3 $GALLSHDB <script/create_db.sql
find $GALLSHDIR -name "*.jp*" -not -path '*THUMB*' >files.csv
find $GALLSHDIR -name "*.png*" -not -path '*THUMB*' >>files.csv
sqlite3 $GALLSHDB <script/import_db.sql

