#!/bin/sh

go build cmd/graftc/main.go
su -c "mv main /usr/bin/graftc && chmod +x /usr/bin/graftc"