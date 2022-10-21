#pragma endian little

struct Data {
	u32 pages_count;
	u32 links_count;
};

Data data @ 0 [[inline]];
