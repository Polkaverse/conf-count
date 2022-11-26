# Conference Monitoring System based on Image Recognition in Rust

This is a Conference Monitoring Project based on Image Recognition that uses Rust Language and AWS Rekognition service to get the level of image similarity. The project can be run on the Raspberry Pi by cross compiling the existing project, details of which are given in the Readme. This project is in the form of Web-Services.

# Project Use-Case

As this project is based on Image Recognition, so here:
* Each attendee's image will be stored in the database at the time of registration for a Conference.
* At the time of the conference, all the images are compared with the clicked image of the conference.
* If a registered attendee is present in the conference then its status will update to *Present* else status will remain *Absent* and a mail is sent to attendee's email address by specifying the absent status.

We thrive for the best and want you to contribute towards a better Project. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for giving your valuable feedbacks and contributions.

## Setting up your environment

### Rustup.rs

Building this project requires [rustup](https://rustup.rs/), version 1.8.0 or more recent.
If you have an older version, run `rustup self update`.

To install on Windows, download and run [`rustup-init.exe`](https://win.rustup.rs/)
then follow the onscreen instructions.

To install on other systems, run:

```
curl https://sh.rustup.rs -sSf | sh
```

This will also download the current stable version of Rust, which this project won’t use.
To skip that step, run instead:

```
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none
```
### MongoDB

#### Download MongoDB

```
wget -qO - https://www.mongodb.org/static/pgp/server-4.2.asc | sudo apt-key add -
sudo apt-get install gnupg
sudo apt-get install -y mongodb-org
```

#### Start MongoDB

```
sudo service mongod start
```


For more information on MongoDB, refer to the MongoDB Manual [`MONGODB MANUAL`](https://docs.mongodb.com/manual/tutorial/install-mongodb-on-ubuntu/)

### Exporting variables

`REGION` is required by AWS which should match your Bucket's region.
You are required to add the same region in which you have your Bucket.

```
export Region=$YOUR_REGION
```

`BUCKET_NAME` is the bucket that stores all the clicked images.

```
export Clicked_Image_Bucket=$BUCKET_NAME
```

`CLICKED_IMAGE_PATH` is the path to the clicked image that is taken by the camera of Raspberry Pi.

```
export Clicked_Image_Path=$CLICKED_IMAGE_PATH
```

`IP_ADDRESS:PORT_NO` refers to the socket where the MongoDB runs.

```
export Host=$IP_ADDRESS:PORT_NO
```

`RUST_LOG` is required for loggers.

```
export RUST_LOG=$Project_Name=Log_Level
```

## Building

### Normal Build

```
git clone https://github.com/Knoldus/conf_count
cd conf_count
cargo build
```

The binary would be saved in `/target/debug/conf_count`

### Cross-compilation build for Raspberry Pi

Run `cargo build --target=armv7-unknown-linux-gnueabihf` to get a cross compiled binary in `/target/armv7-unknown-linux-gnueabihf/debug/conf_count`

## Running the binary

```
./conf_count
```

Then follow the instructions in the application.
