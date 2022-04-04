use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// hostnames, comma separated.
    #[structopt(short, long, default_value = "192.168.66.80,192.168.66.81,192.168.66.82")]
    hosts: String,
    /// ports numbers, comma separated. master:7000, tserver:9000
    #[structopt(short, long, default_value = "7000,9000")]
    ports: String,
    /// update interval (s)
    #[structopt(short, long, default_value = "500")]
    update: u64,
}

fn main() {

    let options = Opts::from_args();
    let hostname_vec: Vec<&str> = options.hosts.split(",").collect();
    let port_vec: Vec<&str> = options.ports.split(",").collect();
    let update_interval: u64 = options.update;

    ybstacksampler::sample_servers( &hostname_vec, &port_vec, update_interval );

}
