Use a Memetic Algorithm with hill climbing to create an optimal keyboard layout

NOTE: The build script assumes you are in a bash shell and have rsync installed

NOTE: The terminal display is not tested on windows.

Notes on the fitness function:
  - Efficiency is not about speed per se. For all of Qwerty's problems, most typing speed records are performed with it. The fitness function is geared toward avoiding uncomfortable hand movement
    - Some amount of SFBs are tolerated if it means avoiding scissors and other unnatural hand motions
  - Redirects are not checked. I don't find them uncomfortable
  - While the home row is favored by penalizing row movement, neither the top nor the bottom row are favored over the other. This avoids placing commonly used keys on the ring or pinky finger to avoid the disfavored row
  - Use of the shift keys is not considered. The way it affects typing is too contextual, both in terms of the specific word being typed and the physical keyboard

Resources:

- Keyboard layouts doc. Crucial for building eval function: https://docs.google.com/document/d/1W0jhfqJI2ueJ2FNseR4YAFpNfsUM-_FlREHbpNGmC2o
- English letter counts. Useful for building a representative corpus: https://norvig.com/mayzner.html
- The best keyboard practice site (IMO): keybr.com
