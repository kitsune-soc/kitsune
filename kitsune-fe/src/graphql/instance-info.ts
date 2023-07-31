import { useQuery } from '@vue/apollo-composable';
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
  const { result }: { result: Ref<{ instance: InstanceInfo } | undefined> } =
    useQuery(gql`
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
    `);

  return computed(() => result.value?.instance);
}

export { useInstanceInfo };
