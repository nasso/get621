/*
 * E6Pool.h
 *
 *  Created on: Jul 16, 2018
 *      Author: nasso
 */

#ifndef GET621_E6POOL_H_
#define GET621_E6POOL_H_

#include <vector>
#include <string>

#include "get621.h"

namespace get621 {
	
	class E6Pool {
		public:
			E6Pool(int32_t id);
			E6Pool();
			virtual ~E6Pool();
			
			void set(const int32_t&);
			E6Pool& operator=(const int32_t&);

			const std::string& json() const {
				return m_json;
			}
			
			time_t createdAt() const {
				return m_createdAt;
			}
			
			const std::string& description() const {
				return m_description;
			}
			
			int32_t id() const {
				return m_id;
			}
			
			bool isActive() const {
				return m_isActive;
			}
			
			bool isLocked() const {
				return m_isLocked;
			}
			
			const std::string& name() const {
				return m_name;
			}
			
			int32_t postCount() const {
				return m_postCount;
			}
			
			const std::vector<get621::E6Post>& posts() const {
				return m_posts;
			}
			
			time_t updatedAt() const {
				return m_updatedAt;
			}
			
			int32_t userId() const {
				return m_userId;
			}

		private:
			friend std::ostream& operator<<(std::ostream&, const E6Pool&);
			
			std::string m_json;

			time_t m_createdAt;
			std::string m_description;
			int32_t m_id;
			bool m_isActive;
			bool m_isLocked;
			std::string m_name;
			int32_t m_postCount;
			time_t m_updatedAt;
			int32_t m_userId;
			std::vector<get621::E6Post> m_posts;
	};

} /* namespace get621 */

#endif /* GET621_E6POOL_H_ */
