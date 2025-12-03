# A list of custom function that the JS runtime provides to Rust

## read_file_for_rust

Reads the content of a network resource and returns it as a base64 encoded string for rust.
Known issue: The function fails for resources larger that a few MBs.

```ts
read_file_for_rust(arg: { callback: (result: string) => void, filename: string })
```

## sleep_for_rust

Wrapper to the setTimeout function to allow sleep in a browser context.
sleep is in ms.

```ts
sleep_for_rust(arg: { callback: (result: string) => void, sleep: number })
```

## exited_did_rust

Signals to the JS runtime that the program finished.
The JS needs to display a message so that the user is not left with a black screen.

## ready_is_rust

Signals to the JS runtime that the program is ready.
The JS needs to stop displaying the loader.

## getScreenSize

Allows rust to get the size of the canvas as it appears on the screen.

## getDrawableScreenSize

Allows rust to get the size of the canvas in px.
