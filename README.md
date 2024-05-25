[![progress-banner](https://backend.codecrafters.io/progress/bittorrent/3375b647-1ec6-41ce-a64a-0b73bd5f31ad)](https://app.codecrafters.io/users/sunmeng90?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own BitTorrent" Challenge](https://app.codecrafters.io/courses/bittorrent/overview).

In this challenge, you’ll build a BitTorrent client that's capable of parsing a
.torrent file and downloading a file from a peer. Along the way, we’ll learn
about how torrent files are structured, HTTP trackers, BitTorrent’s Peer
Protocol, pipelining and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your BitTorrent implementation is in `src/main.rs`. Study
and uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

Time to move on to the next stage!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.70)` installed locally
1. Run `./your_bittorrent.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

## Tricks

* How to convert multiple types of error in method to a single custom error type

  To handle multiple types of errors from within a Rust method and convert them into a single custom error type, you can
  use the anyhow crate. This crate allows you to easily define custom error types that can encapsulate multiple
  underlying
  errors.

* Simplify error
    * anyhow: https://antoinerr.github.io/blog-website/2023/01/28/rust-anyhow.html
    * thiserror

* Wireshark filter to get traffic from specific IP and port

  (ip.src == 165.232.33.77 or ip.src == 178.62.85.20 or ip.src == 178.62.82.89) and (tcp.port == 51467 or tcp.port ==
  51489 or tcp.port==51448)

* trait extension

  Example: `Sink` and `SinkExt`. So, this `impl` block is implementing the `SinkExt<Item>` trait for any type `T` that implements
  the Sink<Item> trait.

```rust

impl<T: ?Sized, Item> SinkExt<Item> for T where T: Sink<Item> {
   
   fn send(&mut self, item: Item) -> Send<'_, Self, Item>
    where
        Self: Unpin,
    {
        assert_future::<Result<(), Self::Error>, _>(Send::new(self, item))
    }
    
}

/// An extension trait for `Sink`s that provides a variety of convenient
/// combinator functions.
pub trait SinkExt<Item>: Sink<Item> {}
```

# Help links

https://danielkeep.github.io/itercheat_baked.html



# debug
cargo run  download_piece -o test-piece-0 sample.torrent 0

connecting to peer [165.232.33.77:51467, 178.62.85.20:51489, 178.62.82.89:51448]

(ip.src == 165.232.33.77 or ip.src == 178.62.85.20 or ip.src == 178.62.82.89) and (tcp.port == 51467 or tcp.port == 51489 or tcp.port==51448)


(ip.src == 165.232.33.77 or ip.src == 178.62.85.20 or ip.src == 178.62.82.89) or (ip.dst == 165.232.33.77 or ip.dst == 178.62.85.20 or ip.dst == 178.62.82.89)
