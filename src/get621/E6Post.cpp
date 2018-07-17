/*
 * E6Post.cpp
 *
 *  Created on: Jun 5, 2018
 *      Author: nasso
 */

#include "E6Post.h"

#include <iostream>

#include "get621.h"

namespace get621 {

	E6Post::E6Post(nlohmann::basic_json<> json) {
		this->set(json);
	}
	
	E6Post::E6Post() {
		
	}
	
	E6Post::~E6Post() {
	}
	
	void E6Post::set(nlohmann::basic_json<> json) {
		m_json = json.dump();
		
		m_id = json["id"];
		m_author = json["author"];
		m_creatorId = json["creator_id"];
		m_createdAt = static_cast<time_t>(json["created_at"]["s"].get<int64_t>());
		
		auto statusStr = json["status"];
		m_status =
			statusStr == "active" ? E6PostStatus::ACTIVE :
			statusStr == "flagged" ? E6PostStatus::FLAGGED :
			statusStr == "pending" ? E6PostStatus::PENDING :
			E6PostStatus::DELETED;
		
		auto sources = json["sources"];
		if(sources.is_array())
			m_sources = sources.get<std::vector<std::string>>();
		
		auto tags = json["tags"];
		m_tags.typed = !tags.is_string();
		
		if(!m_tags.typed) {
			// Tags aren't typed
			m_tags.untyped.clear();
			
			auto untypedTags = tags.get<std::string>();
			
			for(size_t b = 0, i = 0; i < untypedTags.length(); i++) {
				if(untypedTags[i] == ' ') {
					m_tags.untyped.push_back(untypedTags.substr(b, i - b));
					b = i + 1;
				}
			}
		} else {
			// Typed tags
			m_tags.general = tags["general"].get<std::vector<std::string>>();
			m_tags.artist = tags["artist"].get<std::vector<std::string>>();
			m_tags.copyright = tags["copyright"].get<std::vector<std::string>>();
			m_tags.character = tags["character"].get<std::vector<std::string>>();
			m_tags.species = tags["species"].get<std::vector<std::string>>();
		}
		
		m_artists = json["artist"].get<std::vector<std::string>>();
		m_description = json["description"];
		m_favcount = json["fav_count"];
		m_score = json["score"];
		
		auto r = json["rating"];
		m_rating = 
			r == "s" ? E6PostRating::SAFE :
			r == "q" ? E6PostRating::QUESTIONABLE :
			E6PostRating::EXPLICIT;
		
		auto parent_id = json["parent_id"]; 
		m_parentId = parent_id.is_number() ? parent_id.get<int32_t>() : -1;
		
		auto children = json["children"].get<std::string>();
		for(size_t b = 0, i = 0; i < children.length(); i++) {
			if(children[i] == ',') {
				m_children.push_back(std::stoi(children.substr(b, i - b)));
				b = i + 1;
			}
		}
		
		m_hasNotes = json["has_notes"];
		m_hasComments = json["has_comments"];
		m_md5 = json["md5"];
		m_fileUrl = json["file_url"];
		m_fileExt = json["file_ext"];
		m_fileSize = json["file_size"];
		m_width = json["width"];
		m_height = json["height"];
		m_sampleUrl = json["sample_url"];
		m_sampleWidth = json["sample_width"];
		m_sampleHeight = json["sample_height"];
		m_previewUrl = json["preview_url"];
		m_previewWidth = json["preview_width"];
		m_previewHeight = json["preview_height"];
		m_delreason = json["delreason"].is_null() ? "" : json["delreason"];
	}
	
	void E6Post::download(FILE* dest, bool verbose) {
		get621::downloadURL(this->m_fileUrl, dest, verbose);
	}
	
	std::ostream& operator<<(std::ostream& strm, const E6Post& post) {
		strm << "#" << post.m_id << " by ";

		for(size_t j = 0; j < post.m_artists.size(); j++) {
			if(j != 0) strm << (j == post.m_artists.size() - 1 ? " and " : ", ");
			strm << post.m_artists[j];
		}
		
		strm << std::endl << "Rating: ";
		switch(post.m_rating) {
			case E6PostRating::EXPLICIT:
				strm << "Explicit" << std::endl;
				break;
			case E6PostRating::QUESTIONABLE:
				strm << "Questionable" << std::endl;
				break;
			case E6PostRating::SAFE:
				strm << "Safe" << std::endl;
				break;
		}
		
		char timebuf[80];
		strftime(timebuf, 80, "%c", gmtime(&post.m_createdAt));
		
		strm
			<< "Score: " << post.m_score << std::endl
			<< "Favs: " << post.m_favcount << std::endl
			<< "Type: " << post.m_fileExt << std::endl
			<< "Created at: " << timebuf << std::endl;
		
		if(post.m_tags.typed) {
			strm << "Tags:" << std::endl;
			
			if(!post.m_tags.general.empty()) {
				strm << "- General:";
				for(size_t i = 0; i < post.m_tags.general.size(); i++)
					strm << " " << post.m_tags.general[i];
				strm << std::endl;
			}
	
			if(!post.m_tags.artist.empty()) {
				strm << "- Artist:";
				for(size_t i = 0; i < post.m_tags.artist.size(); i++)
					strm << " " << post.m_tags.artist[i];
				strm << std::endl;
			}
	
			if(!post.m_tags.copyright.empty()) {
				strm << "- Copyright:";
				for(size_t i = 0; i < post.m_tags.copyright.size(); i++)
					strm << " " << post.m_tags.copyright[i];
				strm << std::endl;
			}
	
			if(!post.m_tags.character.empty()) {
				strm << "- Character:";
				for(size_t i = 0; i < post.m_tags.character.size(); i++)
					strm << " " << post.m_tags.character[i];
				strm << std::endl;
			}
	
			if(!post.m_tags.species.empty()) {
				strm << "- Sepecies:";
				for(size_t i = 0; i < post.m_tags.species.size(); i++)
					strm << " " << post.m_tags.species[i];
				strm << std::endl;
			}
		} else {
			strm << "Tags (untyped):";
			
			for(size_t i = 0; i < post.m_tags.untyped.size(); i++)
				strm << " " << post.m_tags.untyped[i];
			strm << std::endl;
		}
		
		strm << "Description: " << post.m_description;

		return strm;
	}

	E6Post& E6Post::operator=(const nlohmann::basic_json<>& json) {
		this->set(json);
		
		return *this;
	}
} /* namespace get621 */
