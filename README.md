# Alternative async IO traits
This repo is an experiment in alternative IO traits with several goals in mind:
- Traits should be a good match for the needs of libraries like hyper
- Traits should abstract over both completion-based and readiness-based IO without the need to "if completion do X"
- Traits should allow for memory-saving optimizations like io_uring's provided buffers or buffer pools for the old epoll "only grab a buffer when you are going to read" trick for saving memory
- These should be just as fast for readiness-based IO as the current tokio traits
- Excessive allocation should be avoidable