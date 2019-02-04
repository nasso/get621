/*
 * E6Pool.cpp
 *
 *  Created on: Jul 16, 2018
 *      Author: nasso
 */

#include "E6Pool.h"

#include <iostream>

namespace get621 {
	
	E6Pool::E6Pool(int32_t id) {
		this->set(id);
	}
	
	E6Pool::E6Pool() {
		
	}
	
	E6Pool::~E6Pool() {
	}
	
	void E6Pool::set(const int32_t& id) {
		auto data = get621::endpointGetJSON("/pool/show.json?id=" + std::to_string(id));
		
		if(data == NULL) return;
		
		auto success = data["success"];
		if(success.is_boolean() && !success)
			throw std::runtime_error(data["reason"].get<std::string>());
		
		m_createdAt = static_cast<time_t>(data["created_at"]["s"].get<int64_t>());
		m_description = data["description"];
		m_id = id;
		m_isActive = data["is_active"];
		m_isLocked = data["is_locked"];
		m_name = data["name"];
		m_postCount = data["post_count"];
		m_updatedAt = static_cast<time_t>(data["updated_at"]["s"].get<int64_t>());
		m_userId = data["user_id"];
		m_posts.clear();
		
		nlohmann::basic_json<> json(data);
		
		int32_t page = 1;
		nlohmann::basic_json<> posts = data["posts"];
		do {
			for(auto post : posts) {
				if(page != 1) {
					json["posts"].push_back(post);
				}
				
				E6Post p(post);
				m_posts.push_back(p);
			}
			
			posts = get621::endpointGetJSON("/pool/show.json?id=" + std::to_string(id) + "&page=" + std::to_string(++page))["posts"];
		} while(!posts.empty());
		
		m_json = json.dump();
	}
	
	E6Pool& E6Pool::operator=(const int32_t& id) {
		this->set(id);
		return *this;
	}

	std::ostream& operator<<(std::ostream& strm, const E6Pool& pool) {
		char timebuf[80];
		strftime(timebuf, 80, "%c", gmtime(&pool.m_updatedAt));
		
		strm << "Pool #" << pool.m_id << " by user #" << pool.m_userId << std::endl
			<< "Name: " << pool.m_name << std::endl
			<< "Active: " << (pool.m_isActive ? "Yes" : "No") << std::endl
			<< "Locked: " << (pool.m_isLocked ? "Yes" : "No") << std::endl
			<< "Post count: " << pool.m_postCount << std::endl
			<< "Last updated: " << timebuf << std::endl
			<< "Description: " << pool.m_description;
		
		return strm;
	}

} /* namespace get621 */
