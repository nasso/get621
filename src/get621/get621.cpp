#include "get621.h"

#include <cctype>
#include <iomanip>
#include <sstream>
#include <string>
#include <iostream>
#include <curl/curl.h>

#include "json.hpp"

// curl
static CURL *curl;
static struct curl_slist *hlist = NULL;

static size_t strAppendCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	((std::string*)userp)->append((char*) contents, size * nmemb);
	return size * nmemb;
}

static size_t fileWriteCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	size_t written = fwrite(contents, size, nmemb, (FILE*) userp);
	return written;
}

static std::string url_encode(const std::string &value) {
	std::ostringstream escaped;
    escaped.fill('0');
    escaped << std::hex;

    for (std::string::const_iterator i = value.begin(), n = value.end(); i != n; ++i) {
    	std:: string::value_type c = (*i);

        // Keep alphanumeric and other accepted characters intact
        if (std::isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~') {
            escaped << c;
            continue;
        }

        // Any other characters are percent-encoded
        escaped << std::uppercase;
        escaped << '%' << std::setw(2) << int((unsigned char) c);
        escaped << std::nouppercase;
    }

    return escaped.str();
}

namespace get621 {
	int init() {
		curl = curl_easy_init();
		if(!curl) {
			std::cerr << "Couldn't initialize cURL." << std::endl;
			return 1;
		}
		
		hlist = curl_slist_append(hlist, "User-Agent: get621 (by yann-the-leopard on e621)");
		
		return 0;
	}

	void cleanup() {
		curl_easy_cleanup(curl);
		curl_slist_free_all(hlist);
	}

	// API
	nlohmann::basic_json<> endpointGetJSON(std::string endpoint) {
		CURLcode res;
		std::string readBuffer;
		
	#ifdef NSFW
		curl_easy_setopt(curl, CURLOPT_URL, ("https://e621.net" + endpoint).c_str());
	#else
		curl_easy_setopt(curl, CURLOPT_URL, ("https://e926.net" + endpoint).c_str());
	#endif
		curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
		curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, strAppendCallback);
		curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
		
		res = curl_easy_perform(curl);
		do_sleep(GET621_REQ_COOLDOWN_MS);
		
		if(res != CURLE_OK)
			throw std::runtime_error("Couldn't perform request: " + std::string(curl_easy_strerror(res)));
		
		return nlohmann::json::parse(readBuffer);
	}
	
	int downloadURL(std::string url, FILE* dest, bool printProgress) {
		CURLcode res;
		
		curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
		// curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
		curl_easy_setopt(curl, CURLOPT_NOPROGRESS, printProgress ? 0L : 1L);
		curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, fileWriteCallback);
		curl_easy_setopt(curl, CURLOPT_WRITEDATA, dest);
		
		res = curl_easy_perform(curl);
		do_sleep(GET621_REQ_COOLDOWN_MS);
		
		if(res != CURLE_OK) {
			std::cerr << "Couldn't perform request: " << curl_easy_strerror(res) << std::endl;
			return 1;
		}
		
		return 0;
	}
	
	bool isValidID(char* str) {
		for(int i = 0, l = strlen(str); i < l; i++) {
			if(str[i] < '0' || str[i] > '9') return false;
		}
		
		return true;
	}

	E6Post getPostByID(int32_t id) {
		auto posts = doSearch("id:" + std::to_string(id), 1);
		
		if(posts.empty())
			throw std::runtime_error("Post not found.");
		
		return posts[0];
	}

	std::vector<E6Post> doSearch(std::string search, int32_t limit) {
		auto data = endpointGetJSON("/post/index.json?limit=" + std::to_string(limit) + "&typed_tags=true&tags=" + url_encode(search));
		
		if(data == NULL)
			throw std::runtime_error("Request failed.");
		
		std::vector<E6Post> posts(data.size());
		for(size_t i = 0; i < data.size(); i++)
			posts[i].set(data[i]);
		
		return posts;
	}
}
