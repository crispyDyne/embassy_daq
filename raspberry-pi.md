## Steps for setting up the raspberry Pi
This more or less follows the steps outlined here: [Coding on Raspberry Pi remotely with Visual Studio Code](https://www.raspberrypi.com/news/coding-on-raspberry-pi-remotely-with-visual-studio-code/)

1. Download and install Raspberry [Pi Imager](https://www.raspberrypi.com/software/)
2. Use the imager to install Raspberry Pi OS onto an sd card
    - Configure the host name, user/password, and the wifi settings.
        - You will need to remember the host name and user/password for step 4.
    - See these instructions [How to Set Up a Headless Raspberry Pi](https://www.tomshardware.com/reviews/raspberry-pi-headless-setup-how-to,6028.html) for more details.
3. On your local machine Install VS Code, and the Remote SSH extension for VS Code
4. Start a remote session in VS Code and connect to the raspberry Pi
    - Click on the green box in the lower left of VS Code
    - `Connect to Host`
    - `Add New SSH Host`
    - Enter host name you set in step 2, should look something like: `user@raspberrypi.local`

You should now be remotely developing on your Raspberry Pi!

## Clone this repo onto the Raspberry Pi
1. Open a terminal on the Raspberry Pi (remotely as above or locally)
2. Clone this repo
    - I like to create a `repos` directory in my home directory and clone all my repos there.
    - in the repos directory: `git clone https://github.com/crispyDyne/embassy_daq`
3. create python virtual environment
    - `python3 -m venv ~/envs/daq`
4. activate the virtual environment: 
    - `. ~/envs/daq/bin/activate`
5. install python dependencies
    - `pip install -r pi_can/requirements.txt`

## Setup CAN on the Raspberry Pi
1. Follow the instructions here: [RS485 CAN HAT](https://www.waveshare.com/wiki/RS485_CAN_HAT)

The following is a copy of the instructions from the above link:
```
Using this routine requires first installing the library:
sudo apt-get install python-pip
sudo pip install python-can

Then make sure your mcp2515 kernel driver is open:
sudo vi /boot/config.txt

And add the following: 
dtparam=spi=on
dtoverlay=mcp2515-can0,oscillator=8000000,interrupt=25,spimaxfrequency=1000000

Then restart the raspberry pi：
sudo reboot

send run:
sudo python send.py

Receiving run:
sudo python receive.py

You will see the following:
Timestamp: 1527240783.548918        ID: 0123    S          DLC: 8    00 01 02 03 04 05 06 07
```