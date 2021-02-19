#!/bin/bash

function fml { 
    rlwrap target/release/fml "$@"
}
export -f fml
