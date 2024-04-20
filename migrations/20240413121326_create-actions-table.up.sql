CREATE TYPE action_type AS ENUM ('createmodal');
CREATE TABLE actions (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	slack_id varchar(24),
	slack_user varchar(24),
	slack_channel varchar(24),
	"type" action_type,
	CONSTRAINT actions_pk PRIMARY KEY (id)
);
CREATE INDEX actions_slack_id_idx ON public.actions USING btree (slack_id);
CREATE INDEX actions_slack_user_idx ON public.actions USING btree (slack_user);