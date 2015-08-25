This is a CLI client for the Sumo Logic [Search Job API](https://service.sumologic.com/help/Default.htm#Search_Job_API.htm).  You can use it to search the system and retrieve results.

It's very much a work in progress: in particular, I need to think more about what output formats would be helpful, and about error handling. But it does seem to work in its current form.

To build it, [download Rust](https://www.rust-lang.org) (I used Rust 1.2.0), type `cargo test` to see if it built properly, and then `cargo run -- -h` to see the options.  (When invoking the program, think of `cargo run --` as the name of the program - all flags listed go after that.)  A sample invocation, doing a search for errors by source category over the last 5 minutes, is

```
cargo run -- -u USERNAME -e https://ENDPOINT//api/v1/search/jobs -m 5 "error | count by _sourcecategory"
```
where ENDPOINT is as given in the [API endpoint list](https://service.sumologic.com/help/Default.htm#Sumo_Logic_Endpoints.htm%3FTocPath%3DManage%7C_____11).
