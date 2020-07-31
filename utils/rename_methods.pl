#! /usr/bin/perl


while(<>) {

    s|fmt::Display for ||;
    s|fmt::Result|String|;
    s|f: &mut fmt::Formatter|code: &str|;
    s|fn fmt|fn format_error|;
    s|, err\)|, err.format_error\(\)\)|;
    s|write!|format!|;
    s|(\s+f),||;
    print
}