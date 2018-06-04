const char kPathSeparator =
#ifdef _WIN32
                            '\\';
#else
                            '/';
#endif

// curl
static size_t strAppendCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	((std::string*)userp)->append((char*) contents, size * nmemb);
	return size * nmemb;
}

static size_t fileWriteCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	size_t written = fwrite(contents, size, nmemb, (FILE*) userp);
	return written;
}

static CURL *curl;
static struct curl_slist *hlist = NULL;

int setup_curl() {
	curl = curl_easy_init();
	if(!curl) {
		std::cerr << "Couldn't initialize cURL." << std::endl;
		return 1;
	}
	
	hlist = curl_slist_append(hlist, "User-Agent: get621/0.1 (by yann-the-leopard on e621)");
	
	return 0;
}

void cleanup_curl() {
	curl_easy_cleanup(curl);
	curl_slist_free_all(hlist);
}

nlohmann::basic_json<> getjson_curl(std::string url) {
	CURLcode res;
	std::string readBuffer;
	
	curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
	curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, strAppendCallback);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
	
	res = curl_easy_perform(curl);
	
	if(res != CURLE_OK) {
		std::cerr << "Couldn't perform request: " << curl_easy_strerror(res) << std::endl;
		return NULL;
	}
	
	return nlohmann::json::parse(readBuffer);
}

int download_curl(std::string url, FILE* dest, bool printProgress) {
	CURLcode res;
	
	curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
	// curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
	curl_easy_setopt(curl, CURLOPT_NOPROGRESS, printProgress ? 0L : 1L);
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, fileWriteCallback);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, dest);
	
	res = curl_easy_perform(curl);
	
	if(res != CURLE_OK) {
		std::cerr << "Couldn't perform request: " << curl_easy_strerror(res) << std::endl;
		return 1;
	}
	
	return 0;
}

// API
bool isValidID(char* str) {
	for(int i = 0, l = strlen(str); i < l; i++) {
		if(str[i] < '0' || str[i] > '9') return false;
	}
	
	return true;
}

nlohmann::basic_json<> getPostByID(int id) {
	std::stringstream urlBuilder;

#ifdef NSFW
	urlBuilder << "https://e621.net/post/show.json?id=" << id;
#else
	urlBuilder << "https://e926.net/post/show.json?id=" << id;
#endif
	
	return getjson_curl(urlBuilder.str());
}

nlohmann::basic_json<> doSearch(int tagc, char** tags, bool disableStdout = false) {
	std::stringstream searchQueryBuilder;
	
	bool ordered = false;
	for(int i = 0; i < tagc; i++) {
		searchQueryBuilder << tags[i] << " ";
		
		ordered |= strstr(tags[i], "order:") != NULL;
	}
	
	if(!ordered && tagc < 6) {
		searchQueryBuilder << "order:random";
	}
	
	std::string searchQuery = searchQueryBuilder.str();
	
	std::stringstream urlBuilder;
	
#ifdef NSFW
	if(!disableStdout) std::cout << "E621: " << searchQuery << std::endl;
	urlBuilder << "https://e621.net/post/index.json?limit=1&tags=" << url_encode(searchQuery);
#else
	if(!disableStdout) std::cout << "E926: " << searchQuery << std::endl;
	urlBuilder << "https://e926.net/post/index.json?limit=1&tags=" << url_encode(searchQuery);
#endif
	
	auto data = getjson_curl(urlBuilder.str());
	if(data == NULL) return NULL;
	
	if(data.size() == 0) {
		if(!disableStdout) std::cout << "No post found." << std::endl;
		return NULL;
	}
	
	return data[0];
}
