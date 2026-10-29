# Feed text in the format list-domains.pl / list-users.pl / create-user.pl print
# through virtualmin-gpl json-lib.pl convert_remote_format (as remote.cgi does,
# with multiline forced because json=1 is set) and write the JSON body.
use strict; use warnings;
our %text; our $module_name = "virtual-server";
my ($lib, $cmd, $ex, $in_file, $out_file) = @ARGV;
do $lib or die "load $lib: $@ $!";
open(my $fh, '<', $in_file) or die; local $/; my $out = <$fh>; close $fh;
my $json = &convert_remote_format($out, $ex, $cmd, { 'multiline' => '', 'json' => 1 }, 'json');
open(my $o, '>', $out_file) or die; binmode $o; print $o $json; close $o;
