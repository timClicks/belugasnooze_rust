# Readme

## Build for pi

```bash
# ubuntu [docker]
cross build --target arm-unknown-linux-gnueabihf --release

# alpine docker - armv7-unknown-linux-gnueabihf aka pi zero w
cross build --target arm-unknown-linux-musleabihf --release

cross build --target arm-unknown-linux-musleabihf --release && upx --best --lzma target/arm-unknown-linux-musleabihf/release/belugasnooze -o ./belugasnooze_lite_arm
```

## Cargo watch

```sh
cargo watch -q -c -w src/ -x 'run'
```

## Tests

```sh
cargo test -- --test-threads=1 --nocapture


# Watch for test that start some_prefix
cargo watch -q -c -w src/ -x 'test some_prefix_ -- --test-threads=1 --nocapture'
```

{"name":"add_alarm","body":{"hour":6,"minute":15,"days":[0,1,2,3,4,5,6]}}


{"name":"add_alarm","body":{"hour":11,"minute":49,"days":[5]}}
{"name":"add_alarm","body":{"hour":10,"minute":37,"days":[0,1,2,3,4,5,6]}}


{"name":"add_alarm","body":{"hour":16,"minute":51,"days":[5]}}

{"name":"delete_all"}

{"name" :"delete_one", "body": {"id":5}}

{"name" :"light", "body": {"status":true}}

{"name" :"light", "body": {"status":false}}

{"name" :"time_zone", "body": {"zone":"Europe/Berlin"}}

{"name" :"led_status"}


{"name":"add_alarm","body":{"hour":16,"minute":27,"days":[6]}}