#! /usr/bin/perl

$stat = 0;
while(<>) {
    if (/impl(<'a>)?\s+fmt::Display.+/) {
        $stat = 1;
    }
    
    print unless $stat;

    if(/^\}$/) {
        $stat = 0;
    }
}
