.PHONY: build clean test

build:
	go build -o graftc ./cmd/graftc

clean:
	rm -f graftc
	rm -f *.beam *.erl *.dump

install: build
	su -c "mv graftc /usr/bin/graftc && chmod +x /usr/bin/graftc"

# old target, keeping for reference
#atomvm:
#	./graftc main.gft
#	atomvm main.avm
