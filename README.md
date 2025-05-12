Use a Memetic Algorithm with hill climbing to create an optimal keyboard layout

NOTE: The build script assumes you are in a bash shell and have rsync installed

NOTE: The terminal display is not tested on windows.

Notes on Evaluation Criteria:
  - Efficiency is not about speed per se. For all of Qwerty's problems, most typing speed records are performed with it. The fitness function is geared toward avoiding uncomfortable hand movement
  - Rolls are not checked. If rolls are over-indexed, the algorithm will favor layouts with SFBs and/or other uncomfortable motions. If rolls are given a more reasonable value, they do not appear with any notable frequency among common bigrams, while still exerting a gravity on the overall layout that is hard to reason about
  - Redirects are not checked. I don't find them uncomfortable

Resources:

- Keyboard layouts doc. Crucial for building eval function: https://docs.google.com/document/d/1W0jhfqJI2ueJ2FNseR4YAFpNfsUM-_FlREHbpNGmC2o
- English letter counts. Useful for building a representative corpus: https://norvig.com/mayzner.html
- The best keyboard practice site (IMO): keybr.com
