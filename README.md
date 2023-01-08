# Crowd simulation
## Introduction
The repository content the crowd simulator and some algorithms to localize people in the crowd, detect crowded areas and manage it based on the map of explored area.
## Navigation
###### Root folder
File `/src/main.rs` content basic wave experiment with visualization.
###### Localization experiments
File `/localize_visual_ws/localize_visual/src/main.rs` content localization experiments with visualization. Red nodes are the coordinators.\
File `/localize_ws/localize_ws/src/main.rs` content the same experiments, but optimized without visualization.
###### Crowd detection
File `/crowd_detection/crowd_detection_comp/src/main.rs` content basic experiment with crowd detection, based on the nodes number in transmittion range. Red nodes are the crowded ones.
###### Area exploration
File `/explore/explore/src/main.rs` will content exploration experiment.
