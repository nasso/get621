#include <cstring>
#include <iostream>
#include <curl/curl.h>

#include "get621/get621.h"

const char PATH_SEPARATOR =
#ifdef _WIN32
                            '\\';
#else
                            '/';
#endif

static std::string cwd;

enum class OpMode {
	HELP,
	NORMAL,
	POOL,
	VERSION,
};

enum class PostFilter {
	CHILDREN,
	NORMAL,
	PARENTS,
};

enum class OutMode {
	NORMAL,
	VERBOSE,
	OUTPUT,
	JSON,
};

typedef struct Options {
	OpMode op;
	PostFilter filter;
	OutMode outMode;
	bool save;
	int32_t limit;
	int32_t poolId;
	std::string search;
} Options;

static void parseArgs(int argc, char** argv, Options* opts) {
	opts->op = OpMode::NORMAL;
	opts->filter = PostFilter::NORMAL;
	opts->outMode = OutMode::NORMAL;
	opts->save = false;
	opts->limit = 1;
	opts->poolId = -1;
	
	int tagsStart = 1;
	
	for(int i = 1; i < argc; i++, tagsStart = i) {
			 if(strcmp(argv[i], "--") == 0) { tagsStart++; break; }
		else if(!(strcmp(argv[i], "--children") && strcmp(argv[i], "-c"))) opts->filter   = PostFilter::CHILDREN;
		else if(!(strcmp(argv[i], "--help"    ) && strcmp(argv[i], "-h"))) opts->op       = OpMode::HELP;
		else if(!(strcmp(argv[i], "--json"    ) && strcmp(argv[i], "-j"))) opts->outMode  = OutMode::JSON;
		else if(!(strcmp(argv[i], "--limit"   ) && strcmp(argv[i], "-l"))) opts->limit    = static_cast<int32_t>(std::max(std::min(std::strtol(argv[++i], NULL, 10), 320L), 1L));
		else if(!(strcmp(argv[i], "--output"  ) && strcmp(argv[i], "-o"))) opts->outMode  = OutMode::OUTPUT;
		else if(!(strcmp(argv[i], "--parents" ) && strcmp(argv[i], "-p"))) opts->filter   = PostFilter::PARENTS;
		else if(!(strcmp(argv[i], "--pool"    ) && strcmp(argv[i], "-P"))) opts->op       = OpMode::POOL;
		else if(!(strcmp(argv[i], "--save"    ) && strcmp(argv[i], "-s"))) opts->save     = true;
		else if(!(strcmp(argv[i], "--verbose" ) && strcmp(argv[i], "-v"))) opts->outMode  = OutMode::VERBOSE;
		else if(!(strcmp(argv[i], "--version" ) && strcmp(argv[i], "-V"))) opts->op       = OpMode::VERSION;
		else break; // everything else is considered to be a search tag, so break
	}
	
	// Checks
	if(opts->outMode == OutMode::OUTPUT) opts->limit = 1; // --output only works with a single file 
	if(opts->op == OpMode::POOL) {
		opts->poolId = static_cast<int32_t>(std::stoi(argv[tagsStart], NULL, 10));
	} else {
		std::stringstream strstr;
		
		for(int i = tagsStart; i < argc; i++) {
			if(i > tagsStart) strstr << " ";
			
			strstr << argv[i];
		}
		
		opts->search = strstr.str();
	}
}

static void processPosts(const std::vector<get621::E6Post>& results, const Options& opts, const get621::E6Pool* pool = NULL) {
	std::vector<get621::E6Post> posts;
	
	// Phase 1: filtering posts
	switch(opts.filter) {
		case PostFilter::NORMAL:
			posts = results;
			break;
		case PostFilter::PARENTS:
			for(size_t i = 0; i < results.size(); i++) {
				auto parent = results[i].parentId();
				
				if(opts.outMode == OutMode::VERBOSE) {
					if(parent != -1) std::cout << "#" << parent << " is the parent of #" << results[i].id() << std::endl;
					else std::cout << "#" << results[i].id() << " doesn't have a parent." << std::endl;
				}
				
				if(parent != -1) posts.push_back(get621::getPostByID(parent));
			}
			
			if(opts.outMode == OutMode::VERBOSE) std::cout << std::endl;
			break;
		case PostFilter::CHILDREN:
			for(size_t i = 0; i < results.size(); i++) {
				auto postChildren = results[i].children();
				
				if(opts.outMode == OutMode::VERBOSE) {
					if(postChildren.empty()) std::cout << "#" << results[i].id() << " doesn't have any children." << std::endl;
					else {
						if(postChildren.size() == 1) {
							std::cout << "#" << postChildren[0] << " is the only child of #" << results[i].id() << std::endl; 
						} else {
							std::cout << "Children of #" << results[i].id() << ": #";
							
							for(size_t j = 0; j < postChildren.size(); j++) {
								std::cout << postChildren[j];
								
								if(j < postChildren.size() - 1) std::cout << ", #";
								else std::cout << std::endl;
							}
						}
					}
				}
				
				for(auto child : postChildren) {
					posts.push_back(get621::getPostByID(child));
				}
			}
			
			if(opts.outMode == OutMode::VERBOSE) std::cout << std::endl;
			break;
	}
	
	// Phase 2: output results
	switch(opts.outMode) {
		case OutMode::NORMAL:
			for(auto post : posts) std::cout << post.id() << std::endl;
			break;
		case OutMode::VERBOSE:
			if(pool != NULL) std::cout << *pool << std::endl << std::endl;
			
			if(posts.empty()) std::cout << "No posts matched your search." << std::endl;
			else {
				for(size_t i = 0; i < posts.size(); i++) {
					auto post = posts[i];
					std::cout << post << std::endl;
					
					if(i < posts.size() - 1) std::cout << "--------------------------------" << std::endl;
				}
			}
			break;
		case OutMode::OUTPUT:
			for(auto post : posts) post.download(stdout, false);
			break;
		case OutMode::JSON:
			if(pool != NULL) std::cout << pool->json() << std::endl;
			else {
				std::cout << "[";
				
				for(size_t i = 0; i < posts.size(); i++) {
					auto post = posts[i];
					std::cout << post.json();
					
					if(i < posts.size() - 1) std::cout << ",";
				}
				
				std::cout << "]" << std::endl;
			}
			break;
	}
	
	// Phase 3: save (optional)
	if(opts.save) {
		for(size_t i = 0; i < posts.size(); i++) {
			auto post = posts[i];
			
			std::stringstream pathstr;
			
			if(opts.poolId != -1) pathstr << cwd << PATH_SEPARATOR << opts.poolId << "-" << (i + 1) << "_" << post.id() << "." << post.fileExt();
			else pathstr << cwd << PATH_SEPARATOR << post.id() << "." << post.fileExt();
			
			FILE* destFile = fopen(pathstr.str().c_str(), "wb");
			if(!destFile) {
				if(opts.outMode == OutMode::VERBOSE) std::cerr << "Couldn't open the file: " << pathstr.str() << std::endl;
				continue;
			}
			
			post.download(destFile, opts.outMode == OutMode::VERBOSE);
			fclose(destFile);
		}
	}
}

int main(int argc, char** argv) {
	auto cwdBuf = getcwd(NULL, 0);
	cwd = cwdBuf;
	free(cwdBuf);
	
	Options opts = {};
	parseArgs(argc, argv, &opts);
	
	int returnCode = 0;
	
	switch(opts.op) {
		case OpMode::VERSION:
			std::cout << "get621 - 1.0.0 (by nasso <https://gitlab.com/nasso>)" << std::endl;
			break;
		case OpMode::HELP:
			std::cout
				<< "E621/926 command line tool"
#ifndef NSFW
				<< " (SFW mode)"
#endif
				<< std::endl << std::endl << "  Usage: " << std::endl
				<< "    get621 -h | --help" << std::endl
				<< "    get621 -V | --version" << std::endl
				<< "    get621 [-s] [-c | -p] [-v | -j] -P pool_id" << std::endl
				<< "    get621 [-s] [-c | -p] [-v | -o | -j] [-l limit] [--] [tag...]" << std::endl
				<< std::endl
				<< "  Options:" << std::endl
				
				//      -x, --longer-x               short description                                | 80 characters
				//                                     second line if needed                          | limit
				<< "    -c, --children               Search for children in all the results"            << std::endl
				<< "    -h, --help                   Show this screen"                                  << std::endl
				<< "    -j, --json                   Output JSON info about the posts on stdout"        << std::endl
				<< "    -l, --limit                  Set the post count limit when searching"           << std::endl
				<< "    -o, --output                 Download and output the first post to stdout"      << std::endl
				<< "    -p, --parents                Search for parents in all the results"             << std::endl
				<< "    -P, --pool                   Search for posts in the given pool ID (ordered)"   << std::endl
				<< "    -s, --save                   Download the post to ./<post_id>.<ext>"            << std::endl
				<< "    -v, --verbose                Verbose output about the results"                  << std::endl
				<< "    -V, --version                Print version information and exit"                << std::endl
				<< std::endl;
			break;
		case OpMode::NORMAL:
			{
				get621::init();
				
				try {
					processPosts(get621::doSearch(opts.search, opts.limit), opts);
				} catch(const std::runtime_error& e) {
					std::cout << "Error: " << e.what() << std::endl;
					returnCode = 1;
				}
				
				get621::cleanup();
			}
			break;
		case OpMode::POOL:
			{
				get621::init();
				
				try {
					get621::E6Pool pool(opts.poolId);
					processPosts(pool.posts(), opts, &pool);
				} catch(const std::runtime_error& e) {
					std::cout << "Error: " << e.what() << std::endl;
					returnCode = 1;
				}
				
				get621::cleanup();
			}
			break;
	}
	
	return returnCode;
}
