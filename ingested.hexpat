#pragma endian little
#pragma array_limit    10000000
#pragma pattern_limit 100000000

struct Page {
	u32 link_idx;
//	u32 id;
//	u32 length;
//	bool redirect;
	padding[9];
	u16 title_len;
	char title[title_len];
};

struct Link {
	u32 to;
//	u32 start;
//	u32 end;
	padding[8];
} [[static]];

struct Data {
	u32 pages_len;
	u32 links_len;
	Page pages[pages_len];
	Link links[links_len];
};

Data data @ 0 [[inline]];
