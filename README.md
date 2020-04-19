# Mojang auth checker

A simple tool to check the host file and fix it if it was modified by a "alt account" generator.

## Why ?

Lately I see more and more requests for help on the forums about not being able to login to his Minecraft account. The cause is always the same: a software of "alt account" generator, that modify the hosts file. Being tired of answering the same thing, I developed this software to simplify the task. A second motivation, was to play a little with the rust language.

## How ?

This tool is build in Rust with the [iced](https://github.com/hecrj/iced) GUI library. I'm still a beginner in rust, so the code is not necessarily optimal, but it works ;).
First the tool check if the hosts file is modified. If it's the case, it will be possible to click on the fix button. When the user click on, the tool will try to modify the file, but this will only work if the tool was run as admin. In case of error, the tool will run itself across a powershell that ask the admin right.

## TODO

* [ ] Sign the tool
* [ ] A better icon?
* [ ] Improve the code?
