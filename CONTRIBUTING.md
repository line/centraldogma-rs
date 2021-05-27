## How to contribute to Rust client library for Central Dogma project

First of all, thank you so much for taking your time to contribute! This project is not very different from any other open source projects you are aware of. It will be amazing if you could help us by doing any of the following:

- File an issue in [the issue tracker](https://github.com/line/centraldogma-rs/issues) to report bugs and propose new features and improvements.  
- Ask a question by creating a new issue in [the issue tracker](https://github.com/line/centraldogma-rs/issues).  
  - Browse [the list of previously answered questions](https://github.com/line/centraldogma-rs/issues?q=label%3Aquestion).  
- Contribute your work by sending [a pull request](https://github.com/line/centraldogma-rs/pulls).  

### Run test suite

Run local centraldogma server with docker-compose

```bash
docker-compose up -d
```

Run all tests

```bash
cargo test
```

Run unit test only (centraldogma server not needed)

```bash
cargo test --lib
```


### Contributor license agreement

When you are sending a pull request and it's a non-trivial change beyond fixing typos, please sign [the ICLA (individual contributor license agreement)](https://cla-assistant.io/line/centraldogma-rs).  
Note that this ICLA covers [Central Dogma project](https://github.com/line/centraldogma) and its subprojects, which means you can contribute to [line/centraldogma](https://github.com/line/centraldogma) and [line/centraldogma-rs](https://github.com/line/centraldogma-rs) at once by signing this ICLA.
Please [contact us](mailto:dl_oss_dev@linecorp.com) if you need the CCLA (corporate contributor license agreement).

### Code of conduct
We expect contributors to follow [our code of conduct](https://github.com/line/centraldogma-rs/blob/master/CODE_OF_CONDUCT.md).