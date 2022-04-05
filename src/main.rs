use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// hostnames, comma separated.
    #[structopt(short, long, default_value = "192.168.66.80,192.168.66.81,192.168.66.82")]
    hosts: String,
    /// portnumbers, comma separated.
    #[structopt(short, long, default_value = "7000,9000")]
    ports: String,
    /// update interval (ms)
    #[structopt(short, long, default_value = "500")]
    update: u64,
    /// parallel threads
    #[structopt(long, default_value = "4")]
    parallel: usize,
    /// disable host:port addition
    #[structopt(short, long)]
    disable_hostport_addition: bool
}

fn main() {

    let options = Opts::from_args();
    let hostname_vec: Vec<&str> = options.hosts.split(",").collect();
    let port_vec: Vec<&str> = options.ports.split(",").collect();
    let update_interval: u64 = options.update;
    let parallel: usize = options.parallel;
    let disable_hostport_addition: bool = options.disable_hostport_addition;

    ybstacksampler::sample_servers( &hostname_vec, &port_vec, update_interval, parallel, disable_hostport_addition );

}
