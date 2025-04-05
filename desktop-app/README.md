# Golem GUI Development

## Overview

Golem GUI is a web application that allows users to interact with the Golem network.


## Getting Started

### Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Cargo-make](https://github.com/sagiegurari/cargo-make)
- [Node.js](https://nodejs.org/en/download/) (version 18.x or 20.x LTS recommended)
- [npm](https://www.npmjs.com/get-npm) (included with Node.js)

### Installation

To get started with Golem GUI, follow these steps:

1. Clone the repository:

```bash
git clone https://github.com/golemcloud/golem-cli.git
```

2. Install dependencies:

```bash
cargo make npm-install
```

3. Start the development application (from the root directory):

```bash
cargo make dev-desktop
```

4. Build the application:

```bash
cargo make desktop
```

Application will be opened as window.

## Contributing

We welcome contributions to the Golem GUI project. If you'd like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your changes.
3. Make your changes and commit them.
4. Push your changes to your forked repository.
5. Create a pull request to the main repository.
