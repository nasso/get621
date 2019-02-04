/*
 * E6Post.h
 *
 *  Created on: Jun 5, 2018
 *      Author: nasso
 */

#ifndef GET621_E6POST_H_
#define GET621_E6POST_H_

#include <vector>
#include <string>

#include "json.hpp"

namespace get621 {
	
	enum E6PostStatus {
		ACTIVE,
		FLAGGED,
		PENDING,
		DELETED
	};
	
	enum E6PostRating {
		EXPLICIT,
		QUESTIONABLE,
		SAFE
	};
	
	struct E6PostTags {
		bool typed;
		std::vector<std::string> untyped;
		
		std::vector<std::string> general;
		std::vector<std::string> artist;
		std::vector<std::string> copyright;
		std::vector<std::string> character;
		std::vector<std::string> species;
	};
	
	class E6Post {
		public:
			E6Post(nlohmann::basic_json<> json);
			E6Post();
			virtual ~E6Post();

			void set(nlohmann::basic_json<> json);
			void download(FILE* dest, bool verbose);
			
			E6Post& operator=(const nlohmann::basic_json<>&);
			
			// Getters
			const std::string json() const {
				return m_json;
			}
			
			const std::vector<std::string>& artists() const {
				return m_artists;
			}
			
			const std::string& author() const {
				return m_author;
			}
			
			const std::vector<int32_t>& children() const {
				return m_children;
			}
			
			time_t createdAt() const {
				return m_createdAt;
			}
			
			int32_t creatorId() const {
				return m_creatorId;
			}
			
			const std::string& delreason() const {
				return m_delreason;
			}
			
			const std::string& description() const {
				return m_description;
			}
			
			int32_t favcount() const {
				return m_favcount;
			}
			
			const std::string& fileExt() const {
				return m_fileExt;
			}
			
			int32_t fileSize() const {
				return m_fileSize;
			}
			
			const std::string& fileUrl() const {
				return m_fileUrl;
			}
			
			bool hasComments() const {
				return m_hasComments;
			}
			
			bool hasNotes() const {
				return m_hasNotes;
			}
			
			int32_t height() const {
				return m_height;
			}
			
			int32_t id() const {
				return m_id;
			}
			
			const std::string& md5() const {
				return m_md5;
			}
			
			int32_t parentId() const {
				return m_parentId;
			}
			
			int32_t previewHeight() const {
				return m_previewHeight;
			}
			
			const std::string& previewUrl() const {
				return m_previewUrl;
			}
			
			int32_t previewWidth() const {
				return m_previewWidth;
			}
			
			E6PostRating rating() const {
				return m_rating;
			}
			
			int32_t sampleHeight() const {
				return m_sampleHeight;
			}
			
			const std::string& sampleUrl() const {
				return m_sampleUrl;
			}
			
			int32_t sampleWidth() const {
				return m_sampleWidth;
			}
			
			int32_t score() const {
				return m_score;
			}
			
			const std::vector<std::string>& sources() const {
				return m_sources;
			}
			
			E6PostStatus status() const {
				return m_status;
			}
			
			const E6PostTags& tags() const {
				return m_tags;
			}
			
			int32_t width() const {
				return m_width;
			}

		private:
			friend std::ostream& operator<<(std::ostream&, const E6Post&);
			
			std::string m_json;
			
			int32_t m_id;
			std::string m_author;
			int32_t m_creatorId;
			time_t m_createdAt;
			E6PostStatus m_status;
			std::vector<std::string> m_sources;
			E6PostTags m_tags;
			std::vector<std::string> m_artists;
			std::string m_description;
			int32_t m_favcount;
			int32_t m_score;
			E6PostRating m_rating;
			int32_t m_parentId;
			std::vector<int32_t> m_children;
			bool m_hasNotes;
			bool m_hasComments;
			std::string m_md5;
			std::string m_fileUrl;
			std::string m_fileExt;
			int32_t m_fileSize;
			int32_t m_width;
			int32_t m_height;
			std::string m_sampleUrl;
			int32_t m_sampleWidth;
			int32_t m_sampleHeight;
			std::string m_previewUrl;
			int32_t m_previewWidth;
			int32_t m_previewHeight;
			std::string m_delreason;
	};

} /* namespace get621 */

#endif /* GET621_E6POST_H_ */
