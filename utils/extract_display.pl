#! /usr/bin/perl

$output = "";
$stat = 0;
while(<>) {
    if (/impl(<'a>)?\s+fmt::Display.+/) {
        $stat = 1;
        s/for\s+(\w+)/for semantic_error::$1/;
    }
    
    $output .= $_ if $stat;

    if(/^\}$/) {
        $stat = 0;
    }
}

print $output;
