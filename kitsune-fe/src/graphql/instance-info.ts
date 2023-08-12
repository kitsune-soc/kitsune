import { useQuery } from '@urql/vue';

import gql from 'graphql-tag';
import { ComputedRef, Ref, computed } from 'vue';

type InstanceInfo = {
  description: string;
  domain: string;
  localPostCount: number;
  name: string;
  registrationsOpen: boolean;
  userCount: number;
  version: string;
};

function useInstanceInfo(): ComputedRef<InstanceInfo | undefined> {
  const { data }: { data: Ref<{ instance: InstanceInfo } | undefined> } =
    useQuery({
      query: gql`
        query getInstanceInfo {
          instance {
            description
            domain
            localPostCount
            registrationsOpen
            name
            userCount
            version
          }
        }
      `,
    });

  return computed(() => data.value?.instance);
}

export { useInstanceInfo };
