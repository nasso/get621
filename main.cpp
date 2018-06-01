
#ifdef _WIN32
    #include <direct.h>
	#include <windows.h>
    #define getcwd _getcwd
	#define do_sleep(x) Sleep(x)
#else
    #include <unistd.h>
	#define do_sleep(x) usleep(x * 1000)
#endif

#include <iostream>
#include <curl/curl.h>

#include "json.hpp"
#include "url_encode.hpp"

#include "utils.hpp"

static std::string cwd;

static void printVersion() {
	std::cout << "get621 - 0.1 (by nasso <https://github.com/nasso>)" << std::endl;
}

static void printUsage() {
	std::cout
		<< "Usage: get621 [OPTION] [TAGS]..." << std::endl
		<< "         normal mode (see options below)" << std::endl
		<< "   or: get621 --pool [POOL_ID]" << std::endl
		<< "       get621 -p [POOL_ID]" << std::endl
		<< "         to download all the posts in a pool" << std::endl
		<< "E621/926 command line tool"
#ifndef NSFW
		<< " (SFW mode)"
#endif
		<< std::endl << std::endl
		<< "Options:" << std::endl
		
		//    -x, --longer-x               short description                                | 80 characters
		//                                   second line if needed                          | limit
		<< "  -i, --info                   print info about the first post found" << std::endl
		<< "  -p, --parent                 display info about the parent of the post found" << std::endl
		<< "  -v, --version                output version information and exit" << std::endl
		;
}

static void printPostInfo(nlohmann::basic_json<> post) {
	std::cout << "#" << post["id"] << " by ";

	auto artists = post["artist"];
	for(size_t j = 0; j < artists.size(); j++) {
		if(j != 0) std::cout << (j == artists.size() - 1 ? " and " : ", ");
		std::cout << artists[j].get<std::string>();
	}
	
	std::string rating = post["rating"].get<std::string>();
	
	std::cout << std::endl << "Rating: ";
	if(rating[0] == 's') std::cout << "Safe" << std::endl;
	else if(rating[0] == 'q') std::cout << "Questionable" << std::endl;
	else if(rating[0] == 'e') std::cout << "Explicit" << std::endl;
	else std::cout << rating << std::endl;

	std::cout
		<< "Score: " << post["score"] << std::endl
		<< "Favs: " << post["fav_count"] << std::endl
		<< "Type: " << post["file_ext"].get<std::string>() << std::endl
		<< "Tags: " << post["tags"].get<std::string>() << std::endl;
	
	std::string desc = post["description"].get<std::string>();
	if(!desc.empty()) std::cout << "Description: " << desc << std::endl;
}

static int showParent(int tagc, char** tagv) {
	if(setup_curl() != 0) return 1;
	
	auto post = doSearch(tagc, tagv);
	if(post == NULL) return 1;
	
	do_sleep(500);
	
	auto parent = post["parent_id"];
	
	if(parent.is_null()) {
		std::cout << "#" << post["id"] << " doesn't have a parent." << std::endl;
	} else {
		std::cout << "Parent of #" << post["id"] << ":" << std::endl;
		printPostInfo(getPostByID(parent.get<int>()));
	}
	
	cleanup_curl();
	
	return 0;
}

static int showSearch(int tagc, char** tagv) {
	if(setup_curl() != 0) return 1;
	
	auto post = doSearch(tagc, tagv);
	if(post == NULL) return 1;
	printPostInfo(post);
	
	cleanup_curl();
	
	return 0;
}

static int searchAndSave(int tagc, char** tagv) {
	if(setup_curl() != 0) return 1;

	auto post = doSearch(tagc, tagv);
	if(post == NULL) return 1;
	printPostInfo(post);
	
	auto type = post["file_ext"].get<std::string>();
	std::cout << std::endl << "Downloading to " << post["id"] << "." << type << "..." << std::endl;
	
	// Now save to <id>.<type>
	std::stringstream destPathBuilder;
	destPathBuilder << cwd << kPathSeparator << post["id"] << "." << type;
	
	download_curl(post["file_url"].get<std::string>(), destPathBuilder.str(), true);
	
	cleanup_curl();
	
	return 0;
}

static int savePoolPage(char* poolID, int page, int* counter, int* postCount) {
	std::stringstream urlBuilder;
	
#ifdef NSFW
	urlBuilder << "https://e621.net/pool/show.json?id=" << poolID << "&page=" << page;
#else
	urlBuilder << "https://e926.net/pool/show.json?id=" << poolID << "&page=" << page;
#endif
	
	auto data = getjson_curl(urlBuilder.str());
	
	if(data == NULL) return -1;
	
	auto success = data["success"]; 
	if(success.is_boolean() && !data["success"]) {
		std::cout << "Can't find pool " << poolID << ": " << data["reason"] << std::endl;
		return -1;
	}
	
	if(postCount != NULL) (*postCount) = data["post_count"];
	
	auto poolName = data["name"].get<std::string>();
	std::cout << "Downloading page " << page << " of " << poolName << "..." << std::endl;
	
	auto posts = data["posts"];
	
	for(size_t i = 0; i < posts.size(); i++) {
		auto post = posts[i];
		
		std::cout << "Post " << i << " of this page..." << std::endl;
		
		// Save to i.<type>
		std::stringstream destPathBuilder;
		destPathBuilder << cwd << kPathSeparator << poolID << "-" << ((*counter)++) << "_" << post["id"] << "." << post["file_ext"].get<std::string>();
		
		download_curl(post["file_url"].get<std::string>(), destPathBuilder.str(), false);
	}
	
	return posts.size();
}

static int savePool(char* poolID) {
	if(setup_curl() != 0) return 1;
	
	int postsSaved = 0;
	int postsLeft = -1;
	int page = 1;
	
	int counter = 1;
	
	do {
		std::cout << "Please wait... 3" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 2" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 1" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 0" << std::flush << "\r";
		
		postsSaved = savePoolPage(poolID, page++, &counter, postsLeft < 0 ? &postsLeft : NULL);
		if(postsSaved < 0) return 1;
		
		postsLeft -= postsSaved;
	} while(postsLeft > 0);
	
	cleanup_curl();
	
	return 0;
}

int main(int argc, char** argv) {
	auto cwdBuf = getcwd(NULL, 0);
	cwd = cwdBuf;
	free(cwdBuf);
	
	if(argc == 1 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) printUsage();
	else if(strcmp(argv[1], "--version") == 0 || strcmp(argv[1], "-v") == 0) printVersion();
	else if(strcmp(argv[1], "--info") == 0 || strcmp(argv[1], "-i") == 0) return showSearch(argc - 2, argv + 2);
	else if(strcmp(argv[1], "--parent") == 0 || strcmp(argv[1], "-P") == 0) return showParent(argc - 2, argv + 2);
	else if(strcmp(argv[1], "--pool") == 0 || strcmp(argv[1], "-p") == 0) {
		if(argc < 3 || !isValidID(argv[2])) {
			std::cout << "Please specify a valid pool ID." << std::endl;
			return 0;
		}
		
		return savePool(argv[2]);
	}
	else return searchAndSave(argc - 2, argv + 2);
    
	return 0;
}
