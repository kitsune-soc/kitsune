import { useQuery } from '@urql/vue';

import { computed } from 'vue';

import { graphql } from './types';

function getPostById(id: string) {
  const { data } = useQuery({
    query: graphql(`
      query getPostById($id: UUID!) {
        getPostById(id: $id) {
          id
          subject
          content
          account {
            id
            displayName
            username
            avatar {
              url
            }
            url
          }
          attachments {
            contentType
            description
            url
          }
        }
      }
    `),
    variables: {
      id,
    },
  });

  return computed(() => data.value?.getPostById);
}

export { getPostById };
