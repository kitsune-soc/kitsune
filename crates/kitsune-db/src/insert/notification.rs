use crate::{model::Notification, types::NotificationType};
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(build_method(vis="", name=__build))]
pub struct NewNotification {
    #[builder(default = Uuid::now_v7())]
    pub id: Uuid,
    pub receiving_account_id: Uuid,
    #[builder(default = Timestamp::now_utc())]
    pub created_at: Timestamp,
}

#[derive(TypedBuilder, Clone, Copy)]
struct NewNotificationExtraFields {
    pub triggering_account_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub notification_type: NotificationType,
}

#[allow(non_camel_case_types)]
impl<__id: ::typed_builder::Optional<Uuid>, __created_at: ::typed_builder::Optional<Timestamp>>
    NewNotificationBuilder<(__id, (Uuid,), __created_at)>
{
    pub fn follow(self, triggering_account_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: None,
            notification_type: NotificationType::Follow,
        })
    }

    pub fn follow_request(self, triggering_account_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: None,
            notification_type: NotificationType::FollowRequest,
        })
    }

    pub fn favourite(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Favourite,
        })
    }

    pub fn mention(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Mention,
        })
    }

    pub fn post(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Post,
        })
    }

    pub fn repost(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Repost,
        })
    }

    pub fn post_update(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::PostUpdate,
        })
    }

    fn build(self, extra_fields: &NewNotificationExtraFields) -> Notification {
        let built = self.__build();
        Notification {
            id: built.id,
            receiving_account_id: built.receiving_account_id,
            triggering_account_id: extra_fields.triggering_account_id,
            post_id: extra_fields.post_id,
            notification_type: extra_fields.notification_type,
            created_at: built.created_at,
        }
    }
}
