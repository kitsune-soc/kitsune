import { useQuery } from '@urql/vue';

import { computed } from 'vue';

import { graphql } from '../graphql/types';

function useInstanceInfo() {
  const { data } = useQuery({
    query: graphql(`
      query getInstanceInfo {
        instance {
          description
          domain
          localPostCount
          registrationsOpen
          name
          userCount
          version
          captcha {
            backend
            key
          }
        }
      }
    `),
  });

  return computed(() => data.value?.instance);
}

export { useInstanceInfo };
