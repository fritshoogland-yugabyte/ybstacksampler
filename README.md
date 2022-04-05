# ybstacksampler
This is a utility that collects YugabyteDB thread callstacks via the host:port/threadz endpoints of the YugabyteDB daemons. 
The callstacks are reordered to start with the deepest/first stackframe and end with the latest stackframe:

This is an original callstack from the threadz page:
```
    @     0x7fefdb2addb3  __GI___waitpid
    @          0x37a8677  yb::Subprocess::DoWait()
    @          0x392255d  yb::pgwrapper::PgSupervisor::RunThread()
    @          0x37b66aa  yb::Thread::SuperviseThread()
    @     0x7fefdaddc693  start_thread
    @     0x7fefdb2de41c  __clone
```
Which is in the typical debugger/unwinded order (sometimes seen as function()<-function()<-function() ..etc too).  
Ybstacksampler changes this to:
```
__clone;start_thread;yb::Thread::SuperviseThread();yb::pgwrapper::PgSupervisor::RunThread();yb::Subprocess::DoWait();__GI___waitpid
```
This makes the ordering to be usable for flamegraphs.
(currently I am not including the stack offset, please notify me if there is a need to add it. Lots of tools use function()@hexadecimal address)

The next thing that ybstacksampler does is store and count each hostname:port+callstack combination.  
In this way the stacks can be seen grouped by each individual YugabyteDB daemon.

The utility outputs the hostname:port and callstack combinations with its count:
```
192.168.66.82:9000;__clone;start_thread;yb::Thread::SuperviseThread();std::__1::__function::__func&lt;&gt;::operator()();rocksdb::DBImpl::BackgroundCallFlush();rocksdb::DBImpl::FlushMemTableToOutputFile();std::__1::this_thread::sleep_for();__GI_nanosleep 69
192.168.66.82:9000;__clone;start_thread;yb::Thread::SuperviseThread();yb::ThreadPool::DispatchThread();yb::TaskStream&lt;&gt;::Run();yb::ConditionVariable::WaitUntil();__pthread_cond_timedwait 873
192.168.66.82:9000;__clone;start_thread;yb::Thread::SuperviseThread();yb::ThreadPool::DispatchThread();yb::TaskStream&lt;&gt;::Run();yb::log::Log::Appender::ProcessBatch();yb::log::Log::DoAppend();__writev 1
```
These are collapsed stacks for Brendan Greggs flamegraphs (https://github.com/brendangregg/FlameGraph.git), which with a flamegraph can be created:
[flamegraph](./fg4.svg)

In order to give full visibility to thread activity, all known idle stacks are not printed by the ybstacksampler utility. The known idle stacks can be seen in the source: function sample_servers, HashSet excluded_stacks.

# usage
These are the current options for ybstacksampler:
```
ybstacksampler 0.2.0

USAGE:
    ybstacksampler [FLAGS] [OPTIONS]

FLAGS:
    -d, --disable-hostport-addition    disable host:port addition
        --help                         Prints help information
    -V, --version                      Prints version information

OPTIONS:
    -h, --hosts <hosts>          hostnames, comma separated [default: 192.168.66.80,192.168.66.81,192.168.66.82]
        --parallel <parallel>    parallel threads [default: 4]
    -p, --ports <ports>          portnumbers, comma separated [default: 7000,9000]
    -u, --update <update>        update interval (ms) [default: 500]
```
Typically, only `-h` with the list of hostnames or ip addresses is needed.

In order to capture the output, redirect the output of the ybstacksampler utility to a file:
```
% ./target/release/ybstacksampler -h 192.168.66.80,192.168.66.81,192.168.66.82 > runtest.txt
```
Ybstacksampler will run until it's interrupted via ctrl-c, which will trigger writing the collected stacks.  
With the collected stacks the flamegraph utility can be used directly to create a flamegraph:
```
% ../Flamegraph/flamegraph.pl runtest.txt > runtest.svg
```