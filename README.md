# phupload

> Post a photo to many places automatically

## Features

Upload/post/process your photos to the following places

- Cloudinary
- Custom script
- Flickr (in progress)
- Instagram (in progress)

## Usage

The usage is fairly simple, but requires some setup beforehand.

### Dependencies

`exiftool` is required on your system before this tool can work. It provides access to the EXIF data from your image to automatically know the title, description, camera make, camera model, etc. Installation will depend on your operating system, but you can view the [official installation instructions](https://www.sno.phy.queensu.ca/~phil/exiftool/install.html) for more info.

### Configuration

Add a `config.toml` file to `$HOME/.config/phupload/` to start configuration. This file is in the toml format, which is pretty basic (I think). I may entertain different file types later on!

Here's an example for all publishing types. For more information for Flickr, see the [extra documentation](#flickr) below.

```toml
[cloudinary]
cloud_name = "beardfury"
api_key = "123456789012345"
api_secret = "AbO1cdE2f3gHIjKLMOp4qrstUV5"

[[script]]
path = "/Users/mike/projects/portfolio/add_photo.sh"

[flickr]
oauth_client_key = "a0986b896d0896c0986e0896f0896g89"
oauth_client_secret = "875a9875875d5987"
oauth_access_token = "98398637964097309-a876c876d87f9786"
oauth_access_token_secret = "875876c876c876d8"
```

#### Flickr

In order to use flickr, you must first set up an app in their ["app garden"](https://www.flickr.com/services/). From there, the `key` becomes `oauth_client_key` and the `secret` becomes `oauth_client_secret`.

If you already have an access token and access secret, then you can add those to the configuration as `oauth_access_token` and `oauth_access_token_secret`, respectively. If not, the first time you use the flickr integration, it will create and save them for you.

### Running

To begin the upload process, pass the path to a photo file.

```sh
phupload ./my/photo.jpg
```

## Contibuting

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

Issues and pull requests are welcome!

### Installing

This project is a rust-based project. If you don't have rust yet, I recommend [rustup](https://rustup.rs). Once you have rust and cargo installed, you can start immediately by building the project.

```
cargo build
```

Done! You can test you work by using `cargo run`.

```
cargo run -- ./my/photo.jpg
```

## [License](LICENSE.md)

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore -->
<table>
  <tr>
    <td align="center"><a href="https://www.mike-engel.com"><img src="https://avatars0.githubusercontent.com/u/464447?v=4" width="100px;" alt="Mike Engel"/><br /><sub><b>Mike Engel</b></sub></a><br /><a href="#question-mike-engel" title="Answering Questions">💬</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/phupload/commits?author=mike-engel" title="Code">💻</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/phupload/commits?author=mike-engel" title="Documentation">📖</a> <a href="#example-mike-engel" title="Examples">💡</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/phupload/commits?author=mike-engel" title="Tests">⚠️</a> <a href="#review-mike-engel" title="Reviewed Pull Requests">👀</a> <a href="#maintenance-mike-engel" title="Maintenance">🚧</a> <a href="#design-mike-engel" title="Design">🎨</a> <a href="#infra-mike-engel" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#ideas-mike-engel" title="Ideas, Planning, & Feedback">🤔</a> <a href="#content-mike-engel" title="Content">🖋</a></td>
  </tr>
</table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
