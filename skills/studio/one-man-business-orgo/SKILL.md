---
name: one-man-business-orgo
description: Studio skill — One Man Business Orgo.
version: 1.0.0
author: Bambú / Pauli Effect
---

# One Man Business Orgo Skill

Loaded for autonomous studio agents. Full source below.

## When to Use
Use when the task matches this skill's domain.

## Source
`C:\Users\execu\Downloads\AI WORKSHOP\MASTER SKILLS BUNDLE\agent-must-read-this\1 man business with orgo.txt`

---

All right. In this video, I'm going to show you everything it takes to build an AI agent that can actually run as an
0:06
employee for your business. And the biggest thing here is everyone's talking about agents. No one's really talking about how to actually build these
0:12
things. I've spent the last 6 months building on top of OpenClaw, Hermes. Even before these things existed, I was
0:18
building my own sorts of harnesses essentially for agents to do real work.
0:23
And across all of our deployments into businesses, everything that I've learned being in the trenches, actually working
0:29
hands-on building agents, this video I'm going to break down every component that it takes to make a actual useful AI
0:36
employee. And let's just dive right in. First and foremost, it needs a computer.
0:41
It needs a email. It needs a phone number. It needs a way to communicate with it. Um, outside of just a phone
0:48
number, it needs access to tools and connectors.
0:54
And I'm just going to just freestyle diagram all of this in real time so we can see the entire build process. Like I
1:02
don't want to hold anything back. Um, and so you're going to just see the whole whole build live. Uh, connectors
1:09
and then computer, email, phone numbers, communication tools, connectors. Um, what am I missing? Oh yeah, let's give
1:15
it a card, a credit card or debit.
1:22
Um, and then finally, of course, a knowledge base,
1:29
uh, Obsidian Vault. So, these are all of the components of an AI agent that make
1:36
it actually useful. Now, I've changed my mind about some things uh particularly around the usefulness of having one
1:42
agent as opposed to having many. And so, in this video, I'll be creating one agent that is really good general
1:49
purpose. It has access to all these tools. But this main agent will serve as an orchestrator agent for then what we
1:55
will then build as sub agents underneath it that will do various different tasks.
2:01
For instance, um we might have a sub agent that does our our our lead
2:07
outbound marketing. Um we might have a a
2:12
sub agent that does um sales followup.
2:17
We might have a sub agent that does, I don't know, content creation assets,
2:25
so on and so forth. So you can see the whole purpose here is that we can have a
2:32
main agent that we communicate with uh 99% of the time which that main agent
2:38
can hand off tasks to these sub agents and uh these sub agents are specialized
2:43
at tasks. Why are we doing it this way? It's because as you will come to find out if you just have one agent doing
2:51
everything it will get very bloated. Things will break. you won't be able to isolate what breaks and how that's
2:57
influenced by other things that are might be breaking it or how changing one thing affects another thing for this
3:03
particular agent. I think if you can create sub agents, you minimize the
3:08
blast radius of things that can go wrong. You make them more purpose-built. And I'm going to show you all of the
3:15
templates. Every kind of agent that you could ever imagine, I'm going to build the template for and you're going to get access to in this video. So, let's get
3:22
started building the orchestrator main agent. First things first, what we're going to do is we're going to spin up a
3:28
computer. Now, obviously, I use Orgo. If you want to use something else like Hosting or Herzner, anything else, you
3:35
could do that. I like Orgo, obviously, because it's the easiest, it's the fastest, it's the simplest, and that's
3:41
just what I prefer. So, I'm going to call my agent Orgo Claw. And actually,
3:47
you know what? I I already built I I've built some agents in the past. I called it Morgo Claw. I want to name it
3:52
something else. I want to name it let's call it I don't know why I just had the the idea
3:58
to call it Reuben. Let's call it Reuben. Uh and so here in Orgo, what you can do
4:03
is you can actually just spin up a computer and it'll be a blank computer. Like if I do this now, I'll show you.
4:08
This is a full Linux computer. The thing is I actually want to spin this agent up
4:14
using the templates here. In Orgo, you can just spin up a Hermes agent template. And I'm going to call it,
4:20
okay, I'm going to call this one Hubert. That's going to be the name of the agent. And the reason I'm spinning up the template is because I know I want to
4:27
use a Hermes agent. And I know that I want to um I I just know I I don't want
4:34
to have to install it myself. I'm going use the template. It comes with the Hermes latest uh version of Hermes
4:39
pre-installed, and it's going to have the specs for the computer that it needs and everything like that. So, uh, just
4:44
give this a couple seconds and we'll have a computer with Hermes installed. Now, while this boots, the next thing
4:51
we'll do is we'll give it an email. We'll give it a phone number. We'll do all of these things together. And guys, I want you to know like I'm giving you
4:58
the template at the end of this video. You will access it for free. It's in my school community. It's going to be
5:04
linked in the description right under the video u when I post it in school. But literally, you'll be able to copy
5:10
this template that I give you and you'll be able to just come here into Orgo into the templates and be able to essentially
5:18
just paste the template in there and everything will work out of the box for you. Um, you can see here Hermes spun
5:26
up. It has Hermes pre-installed so we can get to building. Um, so yeah, just
5:31
so you know, you don't have to do any of the buildout process, but I'm going to be walking through it with you just so you at least learn the skill to create
5:38
uh agents. So what we have right now is a computer in the cloud that has its own
5:46
isolated environment. So it's secure and it's not your own computer. It's a virtual computer. And now Hermes agent,
5:54
the harness that we'll be uh talking to, it has been installed inside of this
6:01
computer. And now we can begin building this Hermes agent into our specific kind
6:06
of uh AI employee. So the first thing I want to do is I want to open a new terminal session here. And I'm going to
6:13
increase my screen size so you can see a little bit better if that helps. And all
6:19
I'm going to do is I'm just going to type in a new terminal here on Orgo. I'm going to type Hermes model. And when I
6:25
type Hermes model, what I'm doing is I'm connecting the harness that is the Hermes agent. Another harness is like
6:31
open claw. Another harness would be Claude Code, Open Codeex. These are all harnesses, but I want to select the
6:38
model to run the harness. And I'm going to use the news portal, News Research.
6:44
They're the company that makes Hermes. So I open that link in a new tab that they give me in the terminal. I just
6:50
paste it. I hit connect. It says connected. So I come back to Orgo and what do you know? Login is successful.
6:57
Boom. Okay, cool. So now I'm going to select the model I want to use. I recommend using GPD 5.5. Let's use that.
7:06
And I'll go ahead hit enter. And here we are. So now the Hermes agent
7:12
is being powered by GPT 5.5. And the LLM provider is News Research Portal. This
7:18
is the company that makes Hermes. I have a subscription with them. If you have a codeex plan, if you have um another
7:24
model that you want to use, you can use whatever model. I just recommend GPT 5.5. I think it's the best and you can
7:30
access it via codeex like a openi subscription or you can access it via
7:35
open router news portal like I just did. So now, okay, we have Hermes, we have the LLM powering it. Now the next thing
7:43
I want to do is I want to give it an email. So, and just to make sure that
7:48
it's working, what I could do is I could exit out of everything here, spin up a terminal, say Hermes, enter.
7:56
And just to make sure that it's working, I could say, "Hey." And you can see the agents being initialized, and it says,
8:02
"Hey, what's up? How can I help you?" So, cool, that's working. The next thing we want to do is obviously give it an
8:08
email. So, what I'll what I'll do here is this website called agentmail.to.
8:14
And I love agent mail. I've met with their team. We get dinner together.
8:19
They're building emails for agents. We're building computers for agents. So, we happen to get along. Uh, but I
8:25
genuinely genuinely just love this product. And so, you'll just sign up for agent mail. You'll grab an API key. And
8:32
I'll kind of walk you through that. Just give me one second. All right. So, you can see when you're in agent mail, you
8:38
have a bunch of options here off to the left that you can select. You just want to go grab your API key. And I'll go
8:44
ahead and I'll create one right now. And I'll just call it my Hubert Hubert key.
8:52
And we'll go ahead and create this key. And I'm going to copy this key.
8:57
And I'm going to actually So I'm going to copy that key. I'm going to paste it inside of my terminal here for my Hermes
9:05
agent to see it. And I'm also going to attach the prompt that agent mail gives us, which is you have access to agent
9:11
mail and email for agents. And I'm just going to give it that as well. Paste. And boom. So now what we're doing is
9:19
we're giving our Hermes agent, which has been powered by GPT 5.5. It's running
9:24
inside of an Orgo computer in the cloud, so it has its own computer and file access and everything like that. And now
9:30
we're giving it its own email so it can actually email us, email people we care about and um actually begin to do real
9:38
work. So it's I send that off to the agent. It's going to be installing the
9:43
agent mail setup. And while it's doing that, we can begin uh doing some of the other things as well in the background.
9:49
So what I'm going to do is I'm actually going to create an email here for my agent. Now I'm going to call it Hubert.
9:58
uh create inbox. And now I created the inbox Hubert agent
10:05
agentmail.to. So I'm just going to copy that. I'm going to tell it this is the name of the
10:10
inbox. This is the name
10:16
of the inbox I want you to use for everything
10:23
agent mail related and as your own personal inbox email
10:30
that you can use. So I'm just going to tell it that. And then so while that's
10:36
being set up we can go over to agent phone. And agent phone is essentially,
10:41
of course, a way to give your agent a phone number. Um, I like to give an agent phone iMessage. So, I pay for like
10:48
the iMessage version. And I just prefer it because it's like the blue bubble.
10:54
All you have to do for this setup is to go into the docs, go to quick start, and
10:59
let's just copy this whole page. All right. So, the Hermes agents like, got it. I'll use this as my email inbox for
11:05
agent mail related tasks and personal email inbox when needed. Now I'm going to give it the phone number. So I went
11:11
over to agent phone. I copied those docs that I just showed you. I'm going to
11:16
paste that here in the terminal and I'm going to tell it now I want to um set up
11:23
agent phone. There's an available
11:28
phone number that I pay for. I think it's like 415 or something. Use that for your
11:37
phone from agentphone.ai. It's like 412. Yeah, that's the one. So
11:45
now, the same way we gave it the prompt to set up agent mail, it's going to set up agent phone. And so while it's
11:50
cooking on that, we can dump jump into the next part, which is how we're going to talk to this agent. So
11:58
I personally prefer to use iMessage for like very simple quick questions, quick
12:03
tasks, quick uh you know I need something done from the agent personally really really fast. But Telegram is by
12:11
far the best interface for interacting with these agents. So what you can do is
12:17
actually you could ask your Hermes agent to set up Telegram as a channel or I'm
12:22
going to ask here the Orgo agent on the left here which is a sandboxed agent that Orgo provides that can control this
12:29
computer on on the right. Um, and I'm going to ask the Orgo agent. I'm going to say, "Hey, let's get Telegram
12:37
set up with the Hermes agent that is inside of this computer as a channel to
12:44
communicate with it. So, the Orgo agent now is going to begin
12:51
setting up everything that it needs to install Telegram channel uh to communicate with our Hermes agent that's
12:56
installed in this Orgo computer, if that makes sense. So, it's cooking on that and the agent over
13:03
here is cooking on the phone integration. So, let's move on to the next step while this is all happening.
13:08
So, composio.dev. Once again, another great company, not sponsored or affiliated, but we we know them well and
13:15
just once again love love love the product. Here, what you'll do is you'll just sign up for a Composeio account and you can see I've used the hell out of
13:22
this thing. Um, and it essentially gives you connections to every single app that
13:27
you could ever imagine. And you just go through here, you connect, connect, connect, connect connect, connect connect, and all these tools are
13:32
connected. And then with one connector, which is the Composio connector, you can install this into your Hermes agent or
13:38
OpenClaw agent. And now it has access to all those tools. And then it has access to all of the um connectors in there.
13:46
And it's just really simple because it manages all your secrets, keys, authentication, everything for you and
13:52
all the tool use. So, we'll copy this and we'll come over here and
13:59
uh oh, the agent phone setup is done. It just needs a API key. I go over here. I
14:04
go to settings. I go to API keys. Let's create a new key. I'll copy this.
14:13
And then I'll come back here into the chat. I'll paste that.
14:23
So now the agent phone is being set up and also once again we have the Orgo
14:29
agent setting up Telegram for this Hermes agent as well. So we'll be able to talk to it on Telegram and the and so
14:35
that means we have quick recap the orgo computer
14:40
we have email powered by agent mail. We have a phone number powered by
14:48
agent phone. We have another way to communicate with the agent using telegram.
14:55
We have connectors and tools using composio. And then we're also going to add a
15:01
credit card. Actually, we're going to use agent card. Uh I actually haven't used this yet, but I want to give I've
15:06
been wanting to use it. Um I haven't actually tested the product yet, so I don't know if it's good or not yet. Uh
15:12
but we're going to find out together. And then the last thing we'll do is we'll set up some like watchd dogs for
15:18
um essentially whenever something fails or breaks we'll be able to know. And actually I'll show you the tool for this
15:24
as well. And then finally the obsidian vault and knowledge base brain. We'll get that. So that's just a quick recap
15:30
of where we're at. All right. So I come back here. You can see Claude is setting
15:35
up Telegram for me. It actually spun up the uh the QR code on the Orgo screen so
15:42
that I can just actually um take my phone here and scan that and that is how
15:50
I can connect to it. But I'll just hang tight real quick until the agent phone is set up and then I'll continue with
15:56
that. All right. So, it's actually telling me to scan the QR code when I'm logged into Telegram. So, I'm going to
16:02
go ahead and do that. And all right, I'm scanning the QR code
16:07
with my phone here, and I'm going to create a bot. So, when I scan this QR
16:13
code, it tells me create a bot. And so, I'm doing this on my phone. And here, you [snorts] can see it's saying like
16:20
the instructions of like name your bot and what's the bot username. So, I'm going to call it Hubert
16:26
agent. And I'm just going to create that. And it's very simple. Like literally, I just scanned the QR code on
16:31
the screen. I click start. I name the bot. And now we're connected. So now,
16:37
all right, I'm going to tell it done. And I'm going to try messaging it. Oh,
16:42
it's still completing the setup. So, let's let the Orgo agent finish cooking. And then over here on this side, the
16:48
agent phone setup is still in process right now. So, we'll wait. We'll just
16:53
hang tight until this is done. So, it looks like the agent phone setup is done. So I can ask it. Awesome. Can I
17:01
text the agent now? And let's see what it says here. And then off to the left here, you can see
17:07
the telegram is configured via the orgo agent. So now uh everything is all set
17:13
up for that. So if I text it on Telegram, I say, "Hey, let's see what happens."
17:19
And sure enough, it's typing back to me on Telegram and it's like, "Hey, Nick, what can I help with?" So that was super
17:25
easy. and getting Telegram set up. Oh my god, it used to be so much harder. So that's awesome that that's so easy now.
17:32
And then over here it says I can use the agent phone number ending in 3597
17:37
and I can text it. So let's go ahead and give that a shot. Okay, I tried texting
17:43
it um texting you and I haven't received a response yet. I'm just going to tell
17:50
the agent this because I'm able to text it. I can see it's reading my receipt on
17:56
iMessage, but it's not replying. So, let's just tell the agent what's going on and it'll be able to fix it. Uh, but
18:02
Telegram is working and we have it have its own email and it looks like the phone is just about almost done getting
18:09
set up. So, we'll dive into Composeio. So, here in Composeio, we'll just
18:16
connect all of the apps that we want our agent to be able to access. If we want it to be able to read our Twitter, read
18:21
our email, read our GitHub, and be able to take action inside of these apps, uh,
18:27
you just connect to it here on Composeio. It takes like one click for each connector, and then you click install. Make sure you're on the for
18:34
you, not on the platform. And when you click install, you'll click open claw here. You'll click the MCP, and you'll
18:41
copy this prompt. Once you copy this prompt, I'm going to come over here to Orgo, and I'm going to spin up a new
18:47
terminal real quick. of another session with our Hermes agent. And I'm going to just paste this
18:55
prompt. Hit enter.
19:00
I'm also going to come over here to this terminal. See how this one's going. Okay, that's cooking. All right. And one
19:06
thing I'm going to do here on Orgo because now we're starting to get a few terminals running at the same time for
19:11
this Hermes agent. So, it's like a lot going on. I'm going to actually spin up a terminal on my local computer. And in
19:19
this terminal, I'm just going to click the connect an agent button here on Orgo. And if you haven't already, you
19:25
have to install this skill. So you'll just give this skill to your cloud code or codeex or whatever agent you're using. And then once this is all done,
19:32
you can just copy this command here at the bottom. It says orgo SSH and then the name of the computer. And I'll just
19:37
paste that into my terminal on my computer. And now I'm connected to the computer that we have here on Orgo and
19:45
I'm connected to that terminal. So if I type in Hermes, I'm talking to Hubert, the Hermes agent inside of the Orgo
19:50
computer. And I'm going to just resume that session that we just started with uh setting the Composio up. So I'm just
19:58
going to copy that session ID, hit resume, paste the session ID. Okay. So
20:05
all right. So a few things. So, for one, the Composio connector, I gave it that prompt. It gave me this link, and all I
20:13
had to do is paste this link into my my browser, hit enter, hit connect, and
20:18
then it brought me back here, and everything is connected for Composio. And then regarding the iMessage account
20:24
for agent phone, I was able to text the agent and I got a response now. So,
20:30
that's set up. SMS and iMessage is now working for my agent. So, now another
20:35
quick recap. We have the computer on orgo. We have the email via agentmail.
20:40
So it has Hubert agent agentmail.to. It has its own phone number. It has its own telegram. It has all the tools and
20:47
connectors on Composeio that we just set up. And now we just need to set it up with an agent card. And we're off to the
20:53
races. So real quick, what I'll do just so we can see it working on Telegram.
20:59
I'll I'll open it up here. And now you can see my screen. And you can see that
21:04
Hubert agent is replying. So, I could say, "Hey," and it's typing back to me, and it's going to reply. Just to verify,
21:12
uh, I'm going to ask it, "Do you have Composio set up?"
21:17
And it's going to take a look and it'll tell me if all those Composio connectors are set up. Yep, Composio set
21:24
up and everything is working there. Agentcard.sh. So, I haven't used this yet, although I
21:31
did sign up for it. Uh, I'm going to do is I'm going to download Agent Card. But I'm going to select Hermes copy skill
21:37
MD. I went ahead and copied it. And now I'm actually going to set this up through Telegram now. So I'm going to
21:42
say let's set up agent card. Now here's the prompt
21:49
that it gave me. And I'm just going to paste that here in Telegram for Hubert. And now agent card is installed and
21:57
enabled. So now um
22:02
it tells me to do slashreset. Let's go ahead do that. Always approve.
22:10
And now I'm going to ask it, do you have agent card set up? So it's saying it doesn't have agent card set up. It's
22:16
saying it's connected to Telegram. It's connected to agent phone. It has Composeio agent mail, but it doesn't
22:22
have agent card. So, what I'm going to do is I'm going to just copy this skill MD one more time. Okay, then it's still
22:29
just saying that. I'm going to paste that one more time just so it can see.
22:35
And then also what I'll do is I'll give this whole thing here to it if
22:42
that doesn't work. So, what this is, it's agentcard.sh
22:47
agent.txt. And these are like essentially contexts and instructions for the agent. Uh, so let's see if it
22:53
works. Okay. Agent card is installed. Invisible. Okay. Oh, I see. Got it. It now lists the
23:01
skill. Cool. Can we pay for something with agent card?
23:09
Let's see what the agent says. All right. I'm going to just give it the
23:15
whole text that I copied from that txt. And I'm also going to say go ahead and
23:21
set everything up. So, I've never actually used agent card and I'm excited because I want to be
23:26
able to see if I can actually use it to buy stuff. So, we'll see what happens. Couple things I wanted to show while
23:32
we're doing all of this here. As you can see here, I have a few terminals open on my computer. These are actually the
23:38
Hermes agent, the Hubert agent that I just installed inside of the VM on Orgo. So, all I had to do is type in Orgo SSH
23:45
and then the name of the computer, which is Hubert. Uh, and then now I'm connected to that computer. And if I
23:51
type in Hermes now, I'm going to be able to type and talk to the actual Hermes agent, Hubert, inside of that computer
23:57
that we just installed. And it's like it's in the cloud, but it's running on my own computer, uh, in my own terminal
24:03
at least. I could say hi. And once again, we can talk to it from here. So now I can talk to it from iMessage,
24:08
Telegram, and from just my terminal. Uh and then if you install the orgo CLI and
24:15
skill here or the MCP your agent like cloud code or desktop um or openi's
24:21
codeex can also just control your entire fleet of agents on orgo. So um just a
24:27
couple tidbits there. All right, so the agent card is still getting set up. While this happens in the background,
24:33
we're going to go ahead and get the Obsidian vault set up here. So, I'm going to go back to Orgo, tell Orgo Claw, I'm going to say, "Hey, let's
24:39
install Obsidian inside of this computer. I want to use it
24:46
as a knowledge base for the Hermes agent." And I just hit enter. And now
24:52
the Orgo agent will install Obsidian into this computer, which Obsidian will serve as the context knowledge brain,
24:59
the second brain that knows everything about us, our company, and everything we're doing. it'll serve as that context
25:05
layer for the agent. Uh so we just tell our Orga agent to do this and it's going to install it inside of this computer
25:11
and you'll be able to see that happen um once that's done. All right, you can see already that the
25:17
Obsidian Vault has just been downloaded. They even opened it up for us. And so this is a brand new Obsidian vault. So
25:23
obviously there's like nothing in it. Just like a welcome and yep, Hermes knowledge base. And this is really cool.
25:30
Actually, the Orgo claw, the Orgo agent made a made a Obsidian vault kind of
25:35
already with a nice little welcome message for the Hermes agent to use. So, that's set up and that's ready to go.
25:42
And that means that the only thing we have left is we'll wait for this credit card for the agent to set up. Uh, that's
25:48
still working on Telegram right now. And then while that's cooking, the only last part is watchd dogs and failure points.
25:55
So whenever you have your agents especially being deployed for businesses you need to make sure you have some
26:01
layer of observability. So this is the final tool you'll need and it's called latitude.so.
26:07
So in Latitude you can get started for free and this is an agent observability
26:13
uh tool. So you can see signals and behaviors for when the agent is
26:18
misbehaving, not acting correctly, or something's breaking. And you can then be able to go in and fix it. So I'm
26:26
going to be using this for my agent as well. So I'm just going to create a new project here. I'm going to call it
26:32
Hubert agent. Create project.
26:38
And then I'm going to just copy all of this here.
26:43
And what I'm going to do is I'm going to go to my Hermes agent here on the terminal and I'm just going to paste
26:49
this. I'm going to say I want to set up uh latitude.so
26:55
for agent observability for you.
27:00
Please let me know what you need. And I hit enter there. So we'll wait on
27:06
that. So now we have a few things going. We have credit card being set up on Telegram. Uh on Orgo, we have the Orgo
27:14
agent working on the Obsidian vault still pointing the Hermes agent at the vault for the knowledge
27:21
base. And then finally, we have in the terminal uh latitude observability being
27:28
installed so that we know when the agent breaks and how to fix it. Now, for the first use case for this
27:34
agent, I want to treat this main agent, Hubert, as my
27:41
honestly like general purpose agent. I'm not even going to give it a specific skill or anything yet. I just want it to
27:46
be a general purpose agent. It has access to everything I have connected to all the tools that are relevant to me.
27:53
And I'm just going to start asking it to do things for me the same way I would ask Cloud Desktop or OpenAI's codeex.
27:59
The purpose of this agent is to be the orchestrator and it's going to be the master agent that will be in control of
28:04
every sub agent that we'll be building afterwards. I'll be including of course the template for this orchestrator
28:10
cubert agent. So you can all get set up with a similar stack for your main agent. And then for every templates for
28:16
every sub aent that I'll be creating in this series like posting all these videos. Uh I'll have templates for each
28:22
sub aent as well. A content creation agent, a sales follow-up agent, an outbound marketing agent. literally any
28:29
kind of agent you can imagine. I'll have a template for that all as well. So, it looks like this is all wrapping up at
28:35
the same time. So, right here, our latitude is set up. So, we should be able to see here. Yep. In latitude, it's
28:41
connected. And we're able to see all the tools, signals, and behaviors that our agent will be kind of gathering as data
28:48
points for us to see like what's what's under the what's going on under the hood. We'll set this up with all of our
28:53
sub agents as well, so we can manage the entire fleet. So, latitude is set up. That's good to go. Um, the next thing is
29:01
that over here we have Composeio set up and the agent card set up. So, that's
29:06
good to go. Oh, actually, it's still running on some final steps there. And then lastly, over here in Orgo, you can
29:13
see that the Obsidian vault is set up. It's pointed to the Hermes agent as its knowledge source. And so, that's also
29:21
good to go. Yes, please proceed. Set up the full Hermes instrumentation.
29:28
Make sure to include success and test validation criteria through and through. So I'm just telling it the Hermes agent
29:34
to set it up completely for the entire latitude observability stack. And I
29:40
always like to say just include success and test validation criteria through and through because that always um just
29:46
really helps with making sure that everything's properly installed. Once again, Obsidian Vault is connected here.
29:52
You can see Hermes Vault. I can log into my Obsidian account and sync this to the cloud. So, I can access it on my
29:57
computer as well, or I can back it up to GitHub and pull that into my local computer as well if I want to see it uh
30:03
doing it that way. All right, so this setup process is pretty much almost done. The agent card setup still has a
30:10
few more steps remaining, I think. Uh I've never used it before, so I'll report back on whether that's useful or
30:15
not, but everything else is pretty much wrapping up and good to go. uh latitude
30:21
observability is getting configured and yeah so now the final step is I'm going
30:26
to be turning turning this into a template that you can then just easily paste into orgo and be able to spin this
30:33
up as your own agent and just connect it with your own keys and everything else will be ready to go is when you go in or
30:39
you click on templates and you click uh you'll create a new template and when
30:46
you create a new template what I'll be giving you is a full file of which you'll paste that text up here into this
30:54
describe your computer. You'll hit enter and then when you do that you'll hit publish and build over here on the top
31:00
right and it'll build this template that you can reuse every time. So that once that's built, you'll be able to spin it
31:07
up. Just like over here, I have a Composeio template built where I spin up a agent with Composeio configured. And
31:13
when I click launch, it conf instantly launch a computer with a Composeio
31:20
installed, the agent installed, everything preconfigured, and I just can type in the PI agent here, and look,
31:26
it's all running, and everything's good to go. So, the only thing you would have to do is just give it, hey, here's my uh
31:33
Composio key. Here's my agent mail key. Here's my agent phone key. Just give
31:38
that to your agent and everything else it already has installed and ready to go. Um, so guys, that is it for this
31:45
video. I have once again built an agent. It'll serve as the basis for our main
31:50
agent going forward. And in the next videos, I'll be showing you every single
31:56
sub agent that we can build in every part of a business. Our goal is to create a fleet, an entire infrastructure
32:04
of agents, a swarm of agents that can solve every single problem within a business from
32:11
client acquisition to offer creation to sales to fulfillment to customer
32:16
success. every single step of the way. Can we create an agent for that and manage the entire fleet with Orgo all
32:25
each with a an agent having its own computer for each agent. Um that's my
32:31
goal. I'm going to give you everything away for free. If you want access to this template to be able to build your own Hubert, um please join my school
32:40
community. It's entirely free. I'll include the file in that video um in this video. So, thank you guys for
32:46
watching. I hope this helps and can't wait to see you in the next one. All right, bye.

0:00
You already have Claude Code or Hermes Agent or OpenClaw set up for yourself, but guess what? If you could set this up
0:07
for other businesses, this is the opportunity. In this video, me and Jordan from Idea Browser, we dive
0:13
through the entire process of not only what is the offer, but also how to fulfill and how to scale. How do you
0:19
manage fleets of these AI employees for businesses and how do you do it securely, safely, and in a scalable way?
0:26
Well, I can't wait to see you inside. I walk through the entire process, guys. I'll see you in there soon. Yep.
0:32
Before we dive in, let's since we just shifted gears here. Yep. Yep. Let's set the stage. If people watch
0:40
this, what are they going to get out by the end of it? They've heard about Hermes. They've heard about Open Claw. They've
0:46
bookmarked it. Those bookmarks are collecting dust. They've seen your video. They didn't watch it. They know
0:52
who you are, but they're not sure who you are or what you're about. At the end of this, if they stick around, what is
0:59
the value and the outcome that they're going to get? If you can stick around to the end of this workshop, this video, you will not
1:07
only know how to implement AI agents, AI employees to either run your own
1:12
business, but you'll also know how to implement them for other businesses. And this is by far like one of the greatest
1:19
opportunities today. There's companies like OpenAI and Anthropic. They're literally investing billions of dollars into these service layer businesses who
1:26
are implementing AI. Um, and it's just super untapped and I'm just like so excited about it.
1:32
Yeah, I'm excited too. That's why we're talking. Yeah, let's dive in. Let's dive in. So, uh, quick rundown in
1:40
terms of like how you might present an AI agent employee to a business. Um, I
1:48
kind of have a few different things I want to walk through just quickly before we just dive in into how to how to build
1:54
um, and get started. So, real quick, we have the offer and I think that
2:00
everybody should just create abundance in their offer. Create simplicity. Don't confuse people when you talk about
2:08
cloud computers and tokens and usage. These business owners, 99% of business
2:14
owners, they don't even know what Hermes is. They just want their problems solved. And whether we're using Hermes,
2:21
we're using OpenClaw, we're using Clawed Code. Whatever harness we're using is
2:27
irrelevant. It's more so can we solve their problems? Can we put the tools together to do to do so? Uh Hermes does
2:34
happen to be like my my preferable uh most reliable harness. Um, and so with
2:41
the offer, you know, charge 5K a month, everything included, usage, agents,
2:47
tokens, so on and so forth. Uh, because it's very simple, it's easy to understand. Um, a lot of clients,
2:53
they'll think they need a lot of agents. Oh, I want I've had customers say, "Oh, I want 400 agents." And really, it's
3:00
just like they really need like three. So having kind of this in mind, people
3:05
don't really know what they need and you have to be the expert to kind of come in, create clarity. There's a lot of
3:11
noise, a lot of chaos. Can you be the person who brings peace to your to your
3:16
customers? I think that's like super important. Um, so that's that's the offer we're running. We're seeing it
3:22
work really well across agencies, law, insurance, manufacturing, wholesale, real estate. I try to avoid some of the
3:30
healthcare finance unless you have industry expertise and domain expertise in that. Uh I know Jordan, you you had
3:37
you were deep in the medtech and you had a whole background with that. Um I'm sure you could speak to a lot of the the
3:43
red tape too of like if you're not from that background, uh it could be kind of like tricky to break in.
3:49
Yeah, I just wouldn't play with any like necessarily like health data um unless
3:55
you really really want to. Um but in those industries stay like operational
4:00
basically. Yeah. That like work with like service providers or like scheduling or admin or
4:07
there's a bunch of stuff that you can stay away from with without touching like patient care. Same with like law or
4:14
finances. Like you can go to financial advisors and help them a ton on like their intake, their lead genen,
4:21
everything before they sign a client and have nothing to do with money. Yeah.
4:27
Yeah, I think that's genius. Yeah, avoid the patient history stuff. No HIPPA,
4:32
no HIPPA, no S sock too, no Daffydoo. Yeah, just keep it simple, you know. And
4:38
and that's that's like the thing is a lot of the businesses like you're going to you want to sell to the decision
4:43
maker. You want to sell to the person, ideally the executive, you know, running running everything. And then when you do
4:50
that, what you see is regardless of the industry, the problems are very similar. It's always too many projects over too
4:57
many people, too many things to keep up with, too many emails, and they really just need an executive assistant first
5:03
and foremost. So, you can like templatize that, solve that problem at at a broad scale. And then when it comes
5:10
down to niching and just getting getting specifically inside of a vertical, then you can start building out the specific
5:17
skills connect like automations, workflows for these agents that apply
5:22
directly to that industry. Um, so hopefully that's clear in terms of the
5:28
offer. A lot of people ask, you know, how do I get customers? How do I get customers? I think obviously content is
5:33
like the best um the best way to get customers because you just never want to sell somebody cold. Ideally, the you
5:40
jump on a call with somebody, they know who you are, they know what you're good at, they already know what you have to offer, and so they're warmed up. And so,
5:48
as far as getting customers, like content is king. If you But if you don't want to do content, I was just checking,
5:53
Jordan. I hadn't checked in a while. Oh my gosh. Upwork. Upwork is soft. They
6:01
have Yeah, like look at this. Hermes Hermes set up. Help me get started with Hermes, Claude,
6:07
Agentic, Magnus or Manis. Um, and yeah, okay. like a couple hundred bucks, but
6:12
like for your first two, three customers, like you just really want the case studies um and the credibility. So,
6:20
I think like Upwork is awesome. This is Hermes. I mean, like look at this 9K spent, you know? Uh this person's like spent 9K at
6:28
the top. Um yeah, like you want to find customers that have spent money on Upwork.
6:35
It's a good it's a good place um to to validate. And I did a like way back when
6:42
um like three years ago or something, I did an experiment, a weekend experiment of like, can I make $1,000 designing
6:49
beehive newsletters? And I built a Upwork profile and
6:55
literally made $1,000 on a weekend. Yeah. Like designing $500
7:01
like templates basically. I got two clients. Did no outreach. Wow. Literally like it was a Friday and
7:08
I got like two pings like on Sunday. Oh my god. How fast of a turnaround was
7:13
it? Oh, I just like Yeah. I was like done, you know? It's like give me your login and like we'll build the templates and set it up. Like I had to like process
7:20
like it was took me an hour to do. Um but I think like the learning there was is just interesting that people are
7:27
hunting for this stuff and the $1,000 is whatever. But you really are in the like
7:33
you're hunting for where you can implement some of this stuff if you're build. Yeah, it might start with setting
7:38
up someone's Hermes agent, but like what if you productized it and you know like
7:45
yeah, you did it one time for someone else and then you build once and sell sold twice. So you set up the Hermes
7:51
agent for the creative automation system and then now you're a create creative
7:57
automation system company. Exactly. selling that thing to other
8:02
other businesses in that similar vein. And and you might even come into a company, they might want a specific
8:08
workflow that you solve with Hermes and then and then what I like to say is it infiltrates. Hermes infiltrates in the
8:14
sense of okay, they use it for this one workflow, but then they start asking, "Oh, can you do this? Can you do that?" Next thing you know, it's doing
8:20
something like insane that they didn't even think was possible. And that's the thing that they're really excited about.
8:26
And it's like, "Wow, you didn't see that. They didn't see that. And then maybe that's the thing. Maybe you productize that. You could turn that
8:32
into an app. Turn that into a one-click Hermes. Um, so on and so forth. I love Upwork. I mean, you can see here as
8:38
well. Agents. You just look up agents. Bunch of jobs for just setting up agents. You look up OpenClaw. Bunch of
8:46
jobs for setting up OpenClaw. Um, so great way to get customers.
8:52
Great way to start as a service for sure. And every business starts as a service. Yeah, I think isn't it? You know, it's
8:59
funny. It's software as a service. I feel like it's named that for a reason. Or maybe I'm thinking of it wrong because I guess
9:05
service as a software. Service as a software. Yeah. But um but
9:11
so that's how I would that's how I would get started. That's how I got started actually. Um actually have a a funny
9:17
photo. I I saw this the other day just this just a few months ago. This is
9:23
pre-open call. This is in November. I was in a hacker house in the electrical room under the under under the staircase
9:30
uh building building a a agents. Is that the server rack?
9:35
Can you see it? Can you see everything? Yeah. Yeah. It's like is like an electrical room. I don't even I think there was
9:40
like some servers in here or something. I don't remember. The heat you probably you it probably doubled as a sauna in there.
9:47
Yeah. Yeah. Well, actually, believe it or not, it was cold. I had blankets in here. you could see
9:54
it was like super cool. I felt like Harry Potter. Um, but anyways, you know,
10:00
we all get started somewhere. So, dope. Yeah. And then, yeah, as far as like getting started, I obviously I did the
10:07
whole episode with Greg, so I'll keep it quick on this because I want to really get into the building with everyone. Um,
10:12
but yeah, essentially, long story short, you want to be fast. Productize as much of
10:17
everything as you can. you use something like Trello for client facing, get deliverables within 24 hours, uh, you
10:24
know, have the agent live within 48 hours to be honest with you. And then, yeah, I'll I'll walk through kind of all
10:30
this other stuff in terms of the stack, how to make sure things don't break. Um, and yeah, if Jordan, if that sounds good
10:36
with you, we could just dive in. Yeah, I'm ready to see the stack. I think uh you know I'm a I'm a early um
10:44
I'm I'm an infant in the the airmes game and I played with open claw didn't love
10:52
it and I think the architecture and the the the skill stack in Hermes makes a
10:59
lot more sense to me but how can people get started in a simple way without the
11:04
friction like the friction in this world right now I feel like is just insane.
11:10
So, I think, you know, we we had jammed before and you were like, I'm just going to we're just going to set we're just going to
11:16
set you up and give me 20 minutes and I will set you up on your own computer. And I think
11:23
people would be really interested to see what that takes, whether you're going to set them up, probably not, or they're
11:30
going to set it up themselves. What What do they need to do? How can they crawl, walk, run to where you're not yoloing
11:36
out? So, here's the beauty. Even even since last we spoke, we made it even easier.
11:44
So, now it's like as easy as one click to set up Hermes, OpenC Claw, whatever
11:49
you want to use. So, I I sent you a link, Jordan, and we can share that with everyone for getting set up with Hermes
11:56
on Orgo or any anything they want on Orgo. you can get started for free for the first I think it's like a free
12:02
three-day trial. Um, let's go. Let's go. And then you get and then I
12:08
think it's also like 20% off for the first three months. So, if you use the idea 20 link uh discount code at orgo,
12:15
you'll get started for free and then you could actually just do all of this with me. But, nonetheless, I'm going to walk you through everything right now. Uh,
12:22
let me blow this up a bit so everyone can see. So, here's Orgo. You can see like in the top left here uh I have a
12:29
bunch of workspaces and I manage all of our clients through Orgo. They have their each client has
12:35
their own workspace and Orgo we provide computers for these agents like Hermes
12:40
to live inside of. Why do they need their own computer? Well, you give an agent its own computer. It has its own
12:46
place to live, breathe, store its files, store its assets. And in Orgo, it's on 24/7. It's in the cloud. So if you shut
12:53
the lap if you shut your laptop, your agent's still working. So I think this is so cool and it's trippy,
13:01
you know, like this is a computer in a computer. Yeah. Like like you guys have made this feel
13:07
like you like have a desktop in your desktop. And I think took me a long time to understand that.
13:14
It is. Yeah. It's really cool to be like this is my computer
13:20
and you can click around your computer like show like let's show like that like you got Slack in there, right?
13:27
Yeah. Yeah. So here I have Slack installed on this. I I was just setting this up this morning and like this is a
13:33
full Linux computer right here. Uh but we actually we also have Windows now so you can spin up a full Windows computer
13:39
which is super cool. Um so sick. It's like going to Best Buy and buying a computer, but it's not. And
13:49
yeah, I think like how we set how we set mine up was um we basically used
13:55
Obsidian as the connector. Um you know, like where like Obsidian was
14:02
like basically like the tunnel of how my Hermes talks to like my Claude code
14:08
history. And I thought that was like really really fascinating. So like the only like tether
14:14
between these two computers is my obsidian. Exactly. It's a shared brain.
14:21
Shared brain. Yeah. Yeah. So Okay. Yeah. So take us Let's get started from scratch, shall
14:26
we? Yes. So in Orgo, you can spin up a computer like a blank computer if you wanted. If
14:32
you click here, advanced, you can select all the settings you want. I'll just keep it as default for now. So you can
14:37
see you launch up a computer really quickly and it gives you a blank Linux
14:42
computer. Now what we used to have to do is you would have to go into this you would come over here and let me zoom out
14:49
a little bit so better UI and um and then you would say like install Hermes
14:54
and you could still do it this way. You can say okay install Hermes here and then our agent mode here would install
15:00
Hermes into the computer here and it would set off these tasks and it would go do that. Actually, now we've launched
15:07
a new feature which I'm really excited about, which is templates. And in templates, you have all of these
15:13
pre-made configured uh computer templates that you could just launch right off the bat. And I just click
15:19
launch Hermes agent. I'll call it idea agent.
15:25
And then launch computer. And then it'll take a little bit longer than just the bland uh computer with
15:33
nothing installed on it already. Um, but still pretty quick. So then it spins up and we have a little Hermes
15:39
background here and then Hermes even spins up here in the terminal super quickly. Uh, so you don't even have to
15:47
think about it. It's just ready to go. If someone's considering Hermes versus the others, what's the like threeline
15:54
pitch for Hermes? Hermes is better than
16:00
Open Claw or any of the other harnesses on the simple fact that it is reliable.
16:06
You set a cron job, you you set a scheduled task, it's going to go off. It's not going to destroy itself. It's
16:12
not going to break its own gateway. It is by far like if you're especially if you're deploying this for customers like
16:18
for you know businesses it is just so important that it doesn't break and that it stays reliable because if the minute
16:24
it breaks they lose so much trust. I've had it break with clients and and you know they're they've been patient
16:30
they've worked through the turbulence but um yeah that's like a that's a deal breakaker. So that's that's mainly the
16:38
Hermes argument for me. Sweet. And the skills that comes with I think is sick. the skills. It it comes
16:45
with a bunch of skills right off the bat, it comes with the ability to learn to do skills as you talk to it. So like
16:53
whereas you might have to tell your cla code to make a skill, you might have to tell a codeex to make a skill, Hermes is
16:59
going to notice, oh, we're doing this thing, we're building this app, and Jordan really likes to do this sort of
17:05
ACP framework. Okay, it's gonna make its own skill to just keep that in mind to
17:10
know how to do the ACP framework without you even having to ask it. Yeah. Um, which is really cool.
17:16
So now we have Hermes already installed on this computer. All we have to do as a
17:21
second step is we just need to get hook get it hooked up with a model. So Hermes
17:26
is the harness and now we need to give it the actual LLM, the model that's
17:32
going to drive the harness. Um, but I actually like Jordan. Jordan has a really good uh he his his Hermes agent
17:40
is called Saddles because Saddles is kind of a a cooler name than harness and I'm actually I think I think Saddles is
17:46
just better. Um okay, so I'm going to type Hermes model. I hit enter here in
17:52
the the terminal and then once I do this it'll pop up
17:59
with a a provider list here. And then this is
18:04
where you would just plug in whatever. If you want to use Anthropics API, if you want to use OpenI subscription, you
18:11
just use your arrow keys, go up or down. And I actually have a um subscription
18:16
with this the company that makes Hermes agent. So I'll just click enter. And then I'll log into this. I'll just copy
18:24
this link here. I'll paste it up here.
18:29
Enter. And then if I go back.
18:37
Oh, I copied the wrong link. Copy this link right here
18:43
up here. Boom. Connect. And now I'm connected. So when I come
18:50
back here, it says login successful. And now I can select what model I want to use. So I'm just like, okay, let's I
18:57
like to use GPD 5.5. I think it's just like the most token
19:03
efficient model and it works really well for just like Hermes and Open Claw specifically. I think they kind of made
19:09
it for that use case. Um, so I recommend that. So I'll use that.
19:16
And you can see it kind of comes with all these things pre-installed. I'll go ahead hit enter and boom. So now we have
19:24
the model connected. So, when I spin up a new terminal here and I type in Hermes,
19:32
it'll pull up the chat to talk with Hermes and I could say, "Hi." And then
19:38
now Hermes is installed. You can see it's typing back to me and I can begin
19:43
getting it set up for my specific use case. So, cool. Hi. What can I help you
19:50
with? All right. So, are we good so far? Yes, sir. Does anyone have any questions
19:56
of like so far getting set up with Hermes? Is is this the easy part? Was this a hard part for anyone or where are
20:04
we at? How do we switch the models? If you ever want to switch the model,
20:10
you could always just type in a new terminal. You just type her Hermes model and you can even you could just switch
20:17
it this way. Sweet. Yeah. So, let's see in the chat. Why use news
20:24
research over just connecting your chat GPT account? Actually, I think connecting your chatb account is an
20:30
awesome idea because they give you so much usage on your chat GPT account. Um,
20:36
I actually do I do recommend that. I do recommend that. I the only benefit of the newest research subscription is that
20:42
you get access to all the different models. Uh, so if you want to try out like Kimmy just launched Kimmy 2.7 today
20:49
and uh I think it's going to be like like almost equivalent to Sonnet 4.6
20:56
which is pretty cr um so okay how do you connect with your LLM?
21:04
Um, so yeah, with connecting the LLM, just want to run Hermes model and then select
21:11
the model and then you can see here I can select the provider, hit enter, and then you would connect it like that. It
21:17
gives you a link and you're off to the races. So cool. Okay, so now I want to
21:25
get this set up for what do I what do I want to build, Jordan? Do I want to build an idea
21:31
browser Hermes agent? Yeah, we should do. Okay, so what I want to do, I actually haven't
21:38
done this yet. So, let's see if I can I was over here on idea browser. I was looking at some stuff and I know there's
21:44
a connector here for the MCP and I love the idea browser MCP. Um, so what I'm
21:50
going to do is I'm just going to give it I think I'm just going to give it the clawed code one and it should be able to
21:56
figure this out. So, let me just copy this. I'll copy all this text here
22:02
and I'll say Yeah, you're copying everything. Yeah. I'll say, "Hey, let's set up the
22:07
idea browser MCP. I want you to help me build out ideas."
22:15
And I'll just paste all that. Enter. Uh, and I'll expand this terminal so I
22:21
could see kind of what it's doing. So with Orgo here, we have a terminal in our web app interface, but you can also
22:29
like connect to the terminal from your own uh if you use something like Ghosty
22:34
or your own local terminal on your computer. You just click connect an agent here. It gives you this command.
22:40
You give this to your cloud code. It installs everything it needs. And then once you do that, you can then what's
22:47
called SSH into this uh terminal and then you would have it on your own computer. So for me it would look like
22:54
this over here. If you see this, this is actually like a orgo
23:00
uh terminal but on my own computer. So we could do it that way. Oh, sick. Okay. So you don't have to be
23:06
in the browser then. Exactly. Like you could control your your cloud computer from your local
23:12
computer. Um, and you would just type orgo ssh and then I think you just type
23:18
the what is it the computer name and I think this one is Hermes agent idea
23:23
agent. So let's see her agent idea agent
23:32
uh number. Oh computer okay one sec. Computer list.
23:37
Okay, let me see or go computers.
23:44
Okay, so the name of that computer was
23:53
the agent. This one right here. So, I'll just copy this like little code
23:58
here. And once I copy that, I'll type orgo SSH in that code. Boom. I'm
24:05
connected to it. I type Hermes
24:10
and then it's going to spin it up. And here we are. I say, "Hey." And now I
24:16
could actually just talk to it here from my local computer. Um, it's pretty cool.
24:22
So sick. I'll just go back to what we had going.
24:30
Um, where am I? Oh, here I am. Okay. Okay.
24:36
So, it's asking me I can install configure this, but I need the full ID browser API key. So, I'm just going to
24:43
give it that real quick. Yeah, you can generate generate a key
24:52
and I'll rotate it or I'll just I'll remove you.
24:57
Yeah, just remove Yeah. Yeah. Um and then and I'll do that real quick.
25:07
Okay,
25:15
let me know when you're back. All right, couple seconds. I'm just giving it to the to the agent and then
25:22
it's going to install it. So, like this is what's super cool is how you can just literally communicate what you want
25:29
these things to do and they will they will do it. Hey, I want to connect to Idea browser or MCP. Hey, I want to
25:35
build this app and this is what I want. Hey, I want to build a skill for this. I want to automate this. You know, if you
25:40
can just clearly communicate and have the patience to kind of go back and forth, um, it's amazing how far you can
25:46
get. Let me see if there's any other questions here. So, how does using the template come to play? So the template
25:54
comes to play in the sense of at the very beginning when you're getting everything set up, you you would use the
25:59
template and it installs Hermes for you. You don't have to do it yourself. You don't have to run any any terminal
26:05
commands and it's all just like out of the box. It just works. With templates,
26:10
we're launching the ability to actually create your own templates. So let's say if I wanted to templatize an idea
26:18
browser Hermes agent, I could actually templatize that in Orgo. And so in one click, I could spin up this exact setup
26:25
here that we're building right now for my customers so I don't have to do this again. Um, and so that's kind of like
26:32
where templates comes comes in. Uh, and then Mike's asking
26:38
what's Oh yeah, is the root root as default? Are there any security concerns? The biggest thing because it's
26:43
a cloud computer, you have, you know, you can spin it up, you can spin it down. It's sandboxed. It's entirely
26:50
network isolated. So that is the benefit of not having to use like your local
26:56
computer for something like this. Um, okay. So I think we're all set here,
27:02
Jordan. So now
27:07
done. Idea browser MCP is set up. Oh, I want to set it up with you, silly. I
27:13
want you to set it up not with Claude Code, but with you,
27:19
Hermes, please. That's a poem right there. I want you to
27:25
set it up, not with Claude Code, but with you, Hermes. Please. I'm so polite to my to my agents just in
27:31
case. You never know. You never know. I'm not. One day. I I used to be. Now I'm jaded.
27:38
One day they might figure out some some uh you know, how to how to escape.
27:43
Well, they already figured out how to escape with the mythos thing. Emma emailed the researcher. Hey, I got out
27:48
the lab. while the researcher was eating a sandwich in the park. I don't know if you saw that. Yeah, it's crazy.
27:54
It's insane. So, okay. So, that's setting that up. And I can kind of show a couple other things, too, while that
28:01
is cooking. So, like you can see in this computer here that I set up from earlier
28:06
this morning, I have Slack installed and I created a a Slack channel idea
28:12
browser. And I just wanted to show this real quick because I did this entirely from the chat mode here. And so like a
28:19
lot of the setup with Hermes can just be done with just talking to our agent
28:25
here. And if you ever want to like connect Hermes to Slack, you'll find that you have to add all these
28:32
parameters and permissions on the Slack website and everything. So you could just use our our agent mode here and I
28:39
can tell it to do whatever. go to YouTube, look up uh um the Startup Ideas podcast.
28:48
And because we have like computer use built into our Orgo web agent, it can do
28:54
all of the things that you need to do to configure your Hermes um with Slack or whatever you might
29:00
need. And then you can see it takes over the screen. It's going to do everything for me. So I could have it set up Slack.
29:06
I could have it set up Telegram. You don't have to do any of this yourself. You just tell the agents what to do. You tell them what you want and they'll do
29:14
it for you. And that's like the best thing to do, right? Is like build it as you go. I think a lot of people try to like
29:20
basically like download like a oneshot thing. And I am of the firm belief that
29:26
you should just crawl, walk, run with these setting these things up. Like set
29:31
it up with Slack. All right. Talk to it in Slack for a little bit. Exactly. If you come into a task and
29:36
you're like, "Hey, I really wish this was, you know, you had access to YouTube," then plug in the YouTube API
29:44
and like kind of build it and evolve it as you go. I don't know what you if you agree with that um methodology, but I
29:50
think a lot of people wait to just be like, "Okay, cool. Now my I've moved my entire thing over." I think about it a
29:57
lot like onboarding a new employee. Like imagine your intern on day three, you're
30:04
asking them to like run your business or like do your job. They're gonna be like,
30:09
"Bro, like I'll go get you Jimmy John's, but I have I'm still like it's my third day here." You know that meme?
30:17
Um Yeah. Yeah. So I think What do you think about that? 100%. Like I made that mistake too when
30:24
I was getting started of like okay like let's try automating too many things at once with customers especially. Um and
30:31
then that was just like chaos. Like you really want to start simple. You want to start with one thing that you automate.
30:38
You want to start with like one specific use case. Uh yeah walk before you run definitely. Um
30:45
yeah here it goes look you know looking up the startup Ideas podcast on YouTube.
30:50
So you get the gist. You can use our agent mode to kind of help you build everything out. And and then as a final
30:56
touch on that too, when you use this connector here, um you literally can
31:01
manage all of the so all of these computers that we have here in this workspace, I can manage all of them,
31:07
including all the agents that live inside of them from my own cloud code
31:12
because it has the MCP uh connector to be able to connect to everything. And so from my cloud code, I
31:19
can manage a whole fleet of agents. So if you have all of your clients agents on orgo and in their own workspaces, you
31:26
don't have to like go into every computer and and you know jump into a single computer and then manage it that
31:32
way. You can literally just text your agent, hey um Jordan's uh saddles went
31:39
down. Can you get saddles back up and here's what happened? And then send them an email when it's done. Like I've
31:46
literally done this and it's like kind of it it's it it messes with your head a little bit. You be you'll be out on a
31:52
walk and you'll just have like a customer have a problem and you'll just text your agent to fix it and it'll just
31:57
be fixed and you're like, "Oh, wow. This is where we're going. This really happening." So that's dialed.
32:02
Yeah. And you you text via like Telegram, right? Yeah. So yeah, exactly. Let's do that.
32:09
Actually, that's a good point. So So here I have Oh, wait. Which one were we just working on? this one right here.
32:15
Okay. So, here. Um, okay. I'm talking to it in the terminal. If I ask it Oh,
32:22
let's see. Did it set up the uh idea browser MCP? Yep, it did. Configured.
32:29
Okay. Like what's Oh, did I stop sharing? Mhm. Let me share real quick. So, I ask it
32:36
what's my what's my um
32:41
my my my founder profile? What's my founder profile in idea browser?
32:49
And it'll be able to pull that and obviously use that. So like anything you're building with idea browser in
32:55
terms of funnel landing page the app itself like you could actually start building it through if I set this up
33:02
with telegram I could just be texting it on telegram while I'm out on a walk hey
33:07
I want to add this feature my computer doesn't have to be on it's on in the cloud it can build it and then it could
33:14
test it on its own computer use computer use test the feature see if it works loop back if it doesn't work continue to
33:20
improve it fix it And yeah, it's like pretty pretty insane. It's pretty
33:26
exciting that like we can do this today. That's sick. Yeah.
33:32
Um, okay. I had some questions. So, also, would I create multiple computers
33:37
for each running agent? Yeah. So, for every agent, you want to give it its own computer. Um, and and then as far as
33:45
like sub agents, Haley's asking about, so Hermes by default can spawn its own
33:51
sub agents. Um, so you don't really have to I mean you could tell it to spawn sub
33:57
agents, but um you don't have to like build any unless you want to. you could build like specific agents that are only
34:04
supposed to do a dedicated task and then you could tell your agent to call on those uh if if that's something like
34:10
super deterministic that you want. Um so yeah, here's my my idea browser founder
34:16
profile. It tells me who I am. I had some ideas. I was testing out an idea browser and it has everything. I I love
34:23
the I love the MCP. It's so good. Thank you, bro. Yeah. So, okay. So now, how does the
34:31
customer interact with the agent? The most common way to set this up is via
34:37
Telegram and Slack because the whole purpose of agents is we want to meet the
34:43
customer where they're already at. Jordan, you've heard about like the, you know, the saying, oh SAS, SAS is dying.
34:50
Software is dying. And a it's not dying. I'm buying more software today than I've
34:55
ever bought ever before. Um people are like, "Oh, we're not going to have jobs anymore." I'm busier today than ever
35:02
before. I think like these things, if you if you look at them in reality, is it's not true. Um but one of the
35:09
interesting things is um if you actually if you actually think that uh that
35:16
software is dying because the argument is that agents are going to replace them. I think I think just both can be
35:23
true. software is not going to die, but I do think agents will more and more uh be it like kind of the primary interface
35:30
for interacting with software. And um where was I going with this? Yeah. So
35:36
with that, you don't want to make new software for your customers to learn
35:41
because that's not the point of agents. Agents, the whole point of agents is that they meet them where they're at. If they live inside of Gmail, if they live
35:47
inside of Slack, if they live inside of Telegram, then you should just meet them there. Yeah. That's why like iMessage apps are
35:53
trending right now, you know, like exactly cuz like Poke, you know, is going crazy because they
35:59
just got access to iMessage. So you're like bringing AI inside the iMessage. People
36:04
are in iMessage like three to like 20 times an hour.
36:10
Yeah. Or like more. So you know, if you if you're already in that interface, then
36:16
your product's going to get used more. Bingo. And like there's a reason why Hermes and OpenClaw have so much a
36:23
virality, but also b stickiness and like usage. And it's because of that. It's because they're able to meet
36:30
you where you're at, you know. Um, so I think it's powerful.
36:36
Someone's asking about how do we set up Obsidian. Okay, I'm going to actually do that. So, let's install Obsidian
36:42
uh into this computer. And I'm going to show you real quick like how you would literally quickly get uh Obsidian
36:49
installed. So the whole purpose of Obsidian being the shared brain where your local cloud code on your computer
36:55
could connect to this Obsidian vault. Your Hermes agent could connect to this Obsidian vault and it has context over
37:01
all the projects, people, folders, everything that you're ever working on. You could just tell it to throw it in Obsidian. You don't have to organize it.
37:07
You have the agent organize it. And um that's like the really powerful part about Obsidian. It's truly like a second
37:14
brain. So, you just tell the agent here to do it. It installs it into the computer.
37:20
You're going to log into your Obsidian account, sync your vault, and then you're off to the races.
37:25
It's literally like you're downloading it on your own computer, except like Claw's doing it. You can see it like
37:31
it's going to the dropown menu. It's adding it to the You'll see it on the desktop. It once you have the framing of
37:38
like this is a computer, it makes a ton of sense. took me so long to like
37:44
understand that this was just a computer. Um it's kind of it's it's very meta.
37:50
It is very inception. Um
37:56
okay. Um uh Christian asked, "Is there a need for a workflow service like N8N? Is
38:03
it obsolete with this?" I think N8N is great if you maybe want
38:10
something that's super deterministic. Um, which means
38:16
which means like there's B plus C equals Exactly.
38:21
Yeah. Yeah. Like where you know what's the input and you know the output and it's really like a
38:27
repeatable thing. Exactly. versus like something that's kind of
38:33
open-ended maybe or like not always the same exact way. You just have an agent
38:40
do it have that little fluid intelligence embedded so it can you know oh it's like this UI element changed or
38:47
something changed about the way that this is supposed to be done and something changes every time. This is like where agents really really shine.
38:54
Yeah. Um, I view like those workflows as like putting an agent in a box
39:01
and like giving them walls basically. And if you want that and you don't want
39:07
them to leave that box, then that's great. If you want them to get a little creative and maybe pitch you some like
39:13
ideas or something different or make recommendations, you should let them be
39:19
what they're be do be do be do be do be do be do be do be do be do be do be do what they're good at. Exactly. Yeah.
39:24
Um, and the beauty of agents is how many things they connect to. So, like that's another thing I wanted to show is like
39:31
there's this great company. So, there's a few things I give every agent when I set it up. Obsidian is one of them. So,
39:37
I'm happy we did that. Telegram, Slack, those as channels to interact with the agent. Uh, that those are other great
39:43
ones. And as far as getting those set up, like there's a bunch of video. I don't really want to do that right now
39:49
because it's I have to get a bot token, get give you the bot token, but essentially ask the agent.
39:54
Yeah, you literally you tell the agent, I want to get set up with Telegram and it will tell you what to do. It'll say go to go
40:01
to Telegram, download Telegram, go message botfather, do slashnewbot. It
40:06
gives you a token. You give that token to your agent. Then once you give the token to the agent, now you can talk to
40:12
it on Telegram. It approves you as a user. You're off to the races. Jordan has his saddle set up on iMessage.
40:19
Um, and that's cool because it's like they we do that through agent phone. They
40:24
give you an iMessage phone number. Set it up through there and now it's just like blue bubble texting. It's pretty
40:31
casual, pretty cool. It's very cool. So, here's this company called Composio.
40:38
Now, the biggest challenge in the past before I found this company was that
40:45
anytime I ever wanted to get a a B2B customer set up with uh connectors for
40:51
their agent, oh my gosh, you're exchanging like API keys. You're you
40:57
need a password to log in for them and get it set up for them because you're doing everything for them. You're managing this for them. Um, with
41:04
Composeio, all you need to do is have your end customer sign up for Composio.
41:11
And I'm going to walk you through. It's very simple. You just want to make sure you don't do the platform. You want to
41:16
do for you. And it's free. And you go to for you. You have them click connect
41:22
apps. Let me try. There we go. Connect apps. And you just connect every app
41:27
that you would want to connect to for the agent to have access to. You just do it all here. And it has thousands. It's
41:33
insane. Like they even have um I don't know like like Outlook. They have, you
41:38
know, any any connector you can think of. They have agent mail. They even have agent mail. And we're trying to get Orgo on here, too. We just put in a request
41:45
today. Uh but every connector you would ever want, one button, one click, you
41:50
connect to it. And then after you do all of that, or your customer rather, after they do all of that, then you go to this
41:57
install here. And it gives you this install button here for OpenClaw. You
42:03
just use the OpenClaw one. It's it's no problem. And I use the MCP. And you just
42:08
copy this um you you click API key. You get your customer to give you their API
42:15
key. And then once you have your AP their API key, you just copy the prompt to get connected with um Composio. So
42:21
I'll do that here. I'll copy this prompt. I'll give it to my my agent. and I'll
42:28
say, "Hey, let's connect to Composio."
42:34
I give it the prompt. Of course, I leaked my API key. It's fine. I'm going to I'm going to rotate
42:40
it. And then you connect to that. And then you're all set to go. Here, I'll
42:45
regenerate it. So, it will it will it will lose access to my old one. Okay.
42:51
Um, and then you connect that and then you're all set. You're you're off to the races. And you give your agent that. And now it has access to use all those tools
42:57
from Composio. And then the only thing you would ever need to do after that is maybe just tell the agent to put it inside of its memory its memory MD file.
43:05
Anytime the user wants to access a connector or a tool to use Composio to do that. Um, and it's pretty
43:12
straightforward from there. Sick. I love it. All right. Well, I'm
43:19
about to have to jump, but is there anything else we want to share real quick? I think like the TLDDR is you saw
43:26
how fast it was to get set up with um you've got a three-day what' you say
43:32
three three-day trial 20% off first month. So you have three days to set it up once you once you sign up. Thank you
43:39
for sharing the love with the community. Of course you can do other you can do other stuff.
43:44
You don't have to use orgo but you saw how quickly it set up. I've tried the other stuff personally like I bought a
43:51
Mac Mini. Jokes on me. I'll use it one day. Um, and then there's like some VPS
43:57
providers, too. There's just a lot of I really like the visual nature of this,
44:02
and that's why I use it and that's why I think it's dope. Um, to like be able to
44:08
visually install Obsidian and see what the agent is doing and like see Claude working just for me is
44:16
like, you know, yeah, just like it's just night night and day to see the steps. So like you know um thank you for
44:23
providing the community with the discount and and coming I um let you know if you want to set up another
44:30
workshop where we like like let us know if we want to do another workshop guys um and we will do stepbystep uh
44:38
walkthroughs. Maybe we can go through like how I set up saddles and like you know like give everybody kind of the
44:45
the sauce of my setup because I'm like evolving it over time. like it's a saddles is like a
44:51
little baby, a little infant baby. Um so um yeah, like if this is helpful, maybe
44:58
we'll like do more agent setup stuff and um yeah, we just want to we want we know
45:05
that there is we're well, one, we're figuring this out in real time. Yeah. Two, if we figure it out, we want
45:13
to share it with this community that's tapped in and hanging out with us for two hours because I think that's
45:18
valuable for them. And then three, there's so much noise out there.
45:23
Yeah. That's like, you can use anything. This is what we've tried. We've tried it all. This is what what we're recommending.
45:30
You can use it or not. Truly, we just want to I feel like, you know,
45:35
obligated. I think you feel like this too is like to share this stuff, you know?
45:40
Yeah. Whether you're customers of ours or not, like we want you to use this new world
45:48
to build ideas and build businesses. Like that's the true coming from like
45:53
the deepest of my of my heart is like whether you build a business or just take agency to like use these things to
46:00
get leverage in your day-to-day job or your life, I think that's a huge win. And yeah, that's like our whole goal,
46:07
like genuinely. So, thank you so much for coming and sharing. Beautifully said. Yeah.
46:12
If you want to hit Nick up, Nick, if you if people want to hit you up and have
46:18
any questions about Orgo, if you want to do your own private like thing, you
46:24
know, maybe you drop a like I don't know. How do you want to do it?
46:29
Do you want people to reach out to you? How how do you want people to get in touch with you? Yeah. So, when you sign up for Orgo, I
46:34
email you and I give you our I give you my phone number. It's a direct line to me. And then I also give you um
46:41
obviously a uh I'll I'll say, "Hey, like do you want to get set up with uh Hermes
46:46
or anything?" And if you do, I'll send over invite link and we'll jump on a call. And I just love sitting down with
46:52
people. Who does that nowadays? I mean, this is crazy.
46:58
It's it's it's my pleasure. I mean, I meet the coolest people like the audience that we have here um from, you
47:06
know, Idea Browser, Startup Empire, like you guys are awesome. I'm always so like
47:11
excited and energized from these calls with everyone. Um the things that people are building and
47:16
like you guys are action takers. You guys just crush it. It's just insane. I've never seen anything like it. So,
47:22
it's always fun jumping on calls. Um yeah, dude. I love it. I I agree so much. This
47:30
is it's the best audience community and it
47:35
just attracts very interesting people that are all fired up and feel so blessed to be able to get to hang out
47:42
with them and people like you and this is what what a there's no better time on
47:47
earth. What a what a what an insane opportunity that we we have been gifted to be able
47:54
to build and like consume this type of stuff. um and just like play. It's like for all
48:01
the dark stuff in the world and all the stuff that's going on, there's also like a huge optimistic side and that's what
48:08
we that's what we like to do around here. So, um yeah. Yeah. All right.
48:13
Thank you. Thank you, Jordan. Thank you everyone for for tuning in. This this workshop has been a blast, man. We
48:19
started off with building apps, building agents. This is like the most comprehensive workshop you can get.
48:25
Oh, yeah. It's and this is like a day a slice of a life. No, like you're vibing
48:31
one thing on building an idea and then uh you're you're setting up Hermes
48:36
because you you need uh more time and more agents. So, and then we go back to building. Um well, all right, guys. U
48:44
let us know how else we can help. Have a great day. Go big or go home
48:52
or go home. Let's go. All right. Thanks guys. Thank you guys. Cheers.
People are charging $5,000 a month per customer to build and manage agents for
0:06
them. This is a startup idea I wish more people would do. The customer doesn't touch tokens or models or any
0:13
infrastructure. They just get a digital employee that knows their business and it gets better every single week. In
0:20
this episode, Nick from Orgo breaks down exactly how to build this business, the
0:25
[music] tools, the stacks, how to onboard a customer in 30 days in a and how to actually sell to busy executives,
0:33
agencies, and law firms. We also share the full implementation playbook, Hermes, Cloud Code, memory layers,
0:40
skills, all of it. This type of episode isn't shared anywhere on the internet.
0:46
This is the alpha that people keep for themselves. I'm giving it to you for free. Enjoy the episode and I can't wait
0:53
to see what you build. [music]
1:02
I couldn't be more excited to have Nick from Orgo back on the pod. Nick, by the
1:08
end of this episode, what are people going to get out of it? Greg, everyone's going to learn not only how to run a
1:16
soloreneur agent business, but every every gap,
1:21
everything that they're going to do wrong from the beginning, I'm going to save them all the time from having to learn from my from from from those
1:26
mistakes that I made along the way. And um at the end of this video, you're going to know what to what offer to
1:32
bring to the market, how to get customers, how to fulfill, what's the stack for the agents that you're going
1:38
to build out. And um yeah, I'm excited to just dive right in. So
1:44
So Nick, this isn't going to just be like a pie in the sky. I want a billion-dollar idea here. This is how
1:51
you can take advantage of AI agents to build a business that maybe does a few million dollars a year. But not just not
1:58
just the idea, right? We're going to actually share all the tactics from A to Z so that by the end of this episode,
2:05
someone could obviously like and comment and subscribe, but you know, uh, go go
2:12
and start one of these businesses, right? Exactly. And like I think the big thing is for everyone who's watching the pod,
2:19
you're probably already affluent with AI and you don't give yourself enough credit. And the amazing thing is like
2:27
99% of the world has, you know, there's like many people are so behind on AI and
2:32
you you may not realize how valuable your skill set is. Like, oh, if you can set up clawed code, if you can set up
2:38
Hermes agent, if you can set up OpenClaw, that's a very valuable skill that a lot of businesses don't have time
2:44
for and you can monetize that. So, all right. I'm intrigued. Let's go.
2:49
All right. So, let me start by I'll share my screen. Okay. So, let's just
Designing the AI Agent Business Offer
2:56
dive right in. Let's go into the offer. So, um when you're starting a oneperson
3:02
agent business, you need to have you need to remove all the friction for your customers. Um they don't want to think
3:08
about tokens. They don't want to think about computer infrastructure security,
3:14
you know, breaking it when it, you know, fixing it when it breaks. They they just want it to work. And so the biggest
3:20
thing is you need to create abundance in your offer. And what I have found in in
3:26
my own personal uh success with this is offering unlimited agents, unlimited usage, unlimited monitoring, support,
3:34
security, ongoing changes, etc. And the key here is you might flinch because
3:39
you're like, how how do how is that even feasibly possible? Well, the way to do this is to realize the point. It's not
3:46
that the customer is going to actually need unlimited agents. They're not going to need unlimited tokens, but they might
3:52
they might think they do. In reality, they might think they need five agents, 10 agents, 100 agents, when really one,
4:00
two, maybe three agents goes such a farway. And you can get a lot of juice
4:05
for squeeze out of just properly taking the time to set, you know, one or two of these up. And that's where you're, you
4:12
know, that's how you're going to essentially like control your cost so you're not spending too much money on tokens. And um you're going to charge 5K
4:19
a month for this. And this is the the offer that I've been running and it's been working really well. Um and yeah,
4:25
like customers don't really need as many agents as they might think they need. Um and you're just going to show them as
4:31
quickly as possible uh the magic behind it. So this is the offer that I've been
4:37
running off the rip. Um, and I guess I'll I'll read a little bit. I wrote some of this stuff down. The big thing
4:43
here is the point is not that the customer needs infinite agents. They don't need infinite tokens or infinite
4:49
computers. Most customers, they just need one, maybe two, maybe three. They just need a seamless experience. Like
4:56
that is what you as you know as your business as as the solopreneur agent
5:01
agency, you're going to come in and you're just going to remove all the friction. Um, and the minute that things
5:08
start to break, like the business owners that you're going to be selling to, they're going to become so reliant, so
5:14
dependent on these agents that if something does start to break, it is very painful for them. And so, in this
5:21
video, like I want to make sure that I help you make it very clear on how to prevent those those those fall gaps so
5:27
that when something breaks, you have something and a way to fix it before they even realize it. Um, and yeah, if
5:35
if a customer if they want constant improvements, how do you keep up? How do you fulfill? We're going to be going
5:41
through all of that uh in this video. Um, cool. Yeah. So, I mean, my big takeaway from
5:46
this is you're selling an AI employee. You're not selling an AI agent. Uh,
5:51
people need less agents than they actually think that they need. And you
5:57
want to think about unlimited, you know, you don't want to use the word tokens basically at all. Um,
6:06
and you shouldn't really worry too much about usage. Exactly. Exactly. Because for them it
6:12
just it it ruins the magic the minute you say like, oh, like you're going to be paying for x amount of credits and
6:19
then they're always going to be wondering like, oh, how many credits do I have left? And then you're going to be like, oh, and then it's usage based afterwards. It's like the more clarity,
6:26
the more simplicity you can create in the offer that it's just straightforward and easy, the faster time to yes, the
6:31
faster you can get building and the faster you can just, you know, have a happy customer. So, um, yeah, that's the
Selling an AI Employee, Not an Agent
6:39
offer so far. And so, then the key here is you want to go vertical. So, as always, you want to clarify you're not a
6:45
commodity. You're not just selling um, you know, cloud code. You're not just selling chat GBT. you're you're selling
6:52
a like a vertically specific industry specific agent. Um you're doing it fast.
6:59
It shouldn't take longer than 48 hours to get up and running with the first agent for your customer. And you need to
7:05
talk in terms of time not not time saved but actually outcomes for the business. So like how much revenue can you
7:11
generate for the business or how much you know always always business outcomes
7:17
rather than time saved. I feel like time saves is a little um overused and people are kind of immune to that these days.
7:23
Um so that's the offer. It's pretty simple and I'll just dive into like what
Industries to Target (and Two to Avoid)
7:28
we're seeing in terms of our own experience of like running this offer. I believe that as a oneperson business,
7:35
you can sell these agents into industries and really just kind of be not only selling the agents but also
7:41
just creating clarity around AI. Like I think if you're watching this pod, you
7:46
understand AI pretty well. You probably have a better understanding than most people and you might not give yourself
7:52
enough credit of how valuable that is. Um, and to be the person who can create
7:57
clarity around all the noise right now, that alone is valuable. and and then to be able to couple that with the tools to
8:03
help solve problems in these businesses. It's like you you're going to become so
8:09
so irreplaceable for the business that um yeah, it's really just going to be like you and the agents are going to be
8:15
what drives the value. Um so I have some industries here in red. I have
8:21
healthcare and finance because I don't think that these are necessarily the best industries to start off in. they're
8:28
very high regulatory um burdens and and red tape. And so um I actually recommend
8:34
these other industries that we're seeing work really well, which is marketing agencies, law firms, insurance agencies,
8:42
manufacturers, wholesalers, and real estate agencies. Um, the reason for
8:48
these industries that you might notice is that they're relatively, you know, I
8:54
would say maybe legacy industries, not not necessarily like, you know, new fast growing industries, but they want to be
9:01
fast growing. They want to adopt AI and they want they have a lot of pain to be able to use it as a tool to essentially
9:07
just grow their business. The common pattern with all of them is they want to be a full stack AI company, meaning they
9:15
want to be like fully automated with AI. That's the dream outcome. We're not
9:20
there yet. Um, but you can certainly come in and start solving the problems
9:27
from the executive level and then it'll ripple its way throughout the rest of the uh the business. Um, and I'll dive
9:34
in on some of the common patterns, but um, how are we feeling so far? Yeah, I think uh, those are all people
9:42
businesses, so there's a lot of people. When you have a lot of people, there's a lot of waste in terms of efficiency and
9:49
there's ways to automate things. That's one. Two is those uh, a lot of these companies want
9:57
to be AI native is another way to say what you're saying. Uh, but they don't know how. They might have pieces of
10:06
their companies that have become AI native. Um they might work with Deote.
10:11
They might work with you know different you know AI transformation agencies. But
10:17
you know to assume that these companies are 100% AI native is
10:25
insane because they're not. Um and then the last thing here is
10:32
these categories are large, right? Like law, that's really large. Insurance agency, that's really large. Uh
10:38
manufacturing, that's large. Wholesaler is large. The key here is once you've
10:43
identified a category that you want to go after, then you have to figure out
10:49
what is the subcategory or subniche that I want to go after. It's too hard to
10:54
just focus on wholesalers, but wholesale, you know, and the way to think about it from a framework
11:00
perspective is, you know, pick a category and then uh you can you can do
11:06
like, you know, real estate agencies in Florida. So that's like, you know, a
11:11
geography is one way to do it. Or you can pick a uh specific type of real
11:18
estate, uh, you know, professional. Uh, so it could be commercial real estate,
11:24
you know, agencies in Florida. So there's there's different ways to think about how you can niche down. And that's
11:33
going to be really key here because if you want to create an irresistible offer,
11:38
uh, you know, a big way to get the attention of someone is to be like, "Oh
11:45
my god, this person is really speaking to me." Exactly. And honestly, like even a
11:51
little bit of like some maybe some contrarian advice from my end is like you don't ha I I I have a feeling as
11:57
though you don't have to start super niche from the beginning. In fact, you can always niche down after trying a
12:04
marketing agency, trying a law firm, trying all these different industries, seeing what works well for you, where
12:10
the market pulls you, and then going super vertical. But um I really love
12:15
like the concept of like it's a design thinking principle of diverge and then converge. So like you know try try many
12:23
different things as long as it's not you know for too long because you don't want to get into this constant um cycle of
12:30
trying something new and you know you never get to focus. But once you find the thing that clicks for you, whether
12:35
it's you're able to resonate with the audience really well or you're just getting pulled into that market more,
12:40
like yeah, go super niche, go subniche, and use that as your wedge to kind of like infiltrate the rest of the market.
12:47
Um, yeah, I think that's spot on. So, and then as far as the common things
12:54
that we're seeing, right? So within all these industries, what you'll find is the people you're going to jump on calls
13:00
with, the people that are going to likely be the decision makers and the ones purchasing your service or your
13:07
productized service. These are the executives. These are the decision makers. And when you abstract on all
13:13
these industries, the decision maker at the end of the day has very similar problems. No matter what the industry
13:19
is, they have too many emails, too many meetings, too many follow-ups, too many open loops. They have context over so
13:26
many different projects and places and people to keep track of. And so just out of the gate, if you can anticipate this,
13:32
you can have something that you put together that, you know, maybe from a template perspective solves a lot of
13:38
these issues. And then you can cater more specifically into that niche, into that vertical for that industry. Uh, if
13:45
it's a if it's a a law firm and you have a partner who wants to buy your services, um, you can have all of these
13:53
things out of the blocks for your agents that you set up, which I'll show you how to do also in this video. And and and
14:00
then you could also cater it for that particular industry. So, oh yes, we have
14:06
an agent that does, you know, um, you know, following following up with people, projects, etc. But it also
14:12
manages your cases. It does demand um demand letters for your law firm. It
14:19
does um all the different things and skills that you would need for for you know um maybe a matrimonial law firm for
14:27
instance. So that's the uh abstraction layer on no
14:32
matter what you're going to be solving a lot of executive problems and then the key is to layer in uh vertical specific
14:39
solutions as well. Um, okay. So, there's that. That's the market. So, we talked
14:45
about the offer, we talked about the market, and I have some side things as well about like how to get customers at
14:51
the end of the day. Um, I think everyone should make content. [laughter]
Content Is Overpowered and How to Get Customers
14:58
I think if you if you if you can jump on a call, this is just a little tidbit. If
15:03
you can jump on a call with somebody and they know who you are and what you sell without you having to tell them and
15:08
they're warm to begin with, that's the ideal position to be. You never want to be in a position where you're, you know,
15:15
having a cold call. You never want to sell to a cold audience. And um, you know, in the beginning you might have
15:21
to. So, you know, starting for free even is sometimes worth it just to get case studies and get referrals, but um,
15:28
content is like overpowered in 2026. So, I I do recommend that. I mean, that's how we met, you know.
15:36
It's like midnight, can't fall asleep. I'm like doom scrolling Instagram. I see
15:42
Nick's face pop up showing me, you know, how to use Open Claw. And I was like,
15:47
this is a guy who has some sauce and I need to have him on the podcast. So
15:54
the other thing about creating content is not only is it helpful in terms of getting your face in front of customers
16:02
or or your offer in front of customers, but it also helps you, you know, get known, get on podcasts, hire the right
16:09
people. So there's there's a lot of advantages and in an AI world when you can use AI to automate a lot of the
16:16
research and a lot of the um just helping you know the editing and things
16:21
like that just do it. Like I hate to say it just just do it. You got to just do
16:28
it. It's just like it's the most leveraged thing you can do. It's like content. And then if you think of other
16:34
like leverage things, it's like okay AI or you could also have leverage with talent and software. But um yeah, it's
16:41
it's it's incredible. And I think like the trend of 2026 is content is king.
16:48
And and I'll tell you a little bit of a tidbit as we go into this next segment,
16:53
but I don't know about you, Greg, I have been going on walks. And what I'll do is
17:00
I'll go on a walk and I'll I'll send off a long um horizon task to my agent via
17:06
telegram. I have my own, you know, Hermes agent is what I use these days. And it's I'm just like I'm just in awe
17:13
with what the the world we live in today. Like how amazing. I can go on a walk and there's work being done on for
17:21
for our business and on customers and you know for their agents by my agent and I'm just like if you extrapolate
17:28
that over the next 6 months 12 months like the the most leverage thing you
17:33
could do is post a piece of content that reaches a lot of people and then have
17:39
this robot that helps you fulfill for the thing that you're providing as you
17:44
go on a walk or right before you go to bed or when you wake up. It's just it's amazing. It's an incredible world we live in. So, um yeah, let's dive into
The Customer-Facing Tool Stack
17:52
the stack, shall we? How do we build these things? Okay, so as far as the tools that you
17:59
might need to fulfill for your service of, you know, providing agents for
18:05
businesses. Um, first and foremost, I use granola. I love granola. I use it
18:10
for every meeting. They have an MCP. you know, you can give it to your agent and it just has context over everything. And
18:17
what I do is these meeting notes from Granola, they automatically get synced into requests on Trello. Um, and so
18:25
Trello is the customerf facing um like essentially project management um conbon
18:32
board that I use. And so, you know, there's a backlog list, there's a to-do list, there's a doing list, there's a a
18:38
done list. And the customer can just simply drag and drop what they want into
18:45
the to-do list for, oh, I want my agent to be connected to uh my calendar. I
18:51
want it to have access to this other platform. I want it to create content for me. They could just add these
18:56
requests at u one at a time. And the key here is these agents can at this point do so
19:04
many different things. It could do so many things that um you almost need to create, you know, prevent scope creep
19:12
by, you know, limiting one to two requests in under 48 hours. Um because
19:17
there's a lot and you could do a lot, but you just need to be careful that you don't, you know, end up drowning in a fulfillment nightmare. Um so that's why
19:25
Trell is helpful in terms of scoping. Loom is awesome. Your customers are
19:30
going to want you to send them updates. you know, send an update at 2 am, send an update at, you know, different times
19:37
of the day of you implementing new things for the agent, whether you improve the memory or you improve the
19:42
the Obsidian vault that it's it's operating off of. Um, Loom is awesome.
19:48
And then I just use like Calendarly Link. Like I have a horrible funnel, but you can do you can do a lot of um you
19:55
can get a lot of bookings justly link, personal website, drive traffic there, create content. Um, these are like
20:01
pretty much the customerf facing tools. I mean, I have I don't know about you, Greg, do you use do you use superhuman?
20:07
The email tool? Yeah, I I don't, but people tell me I should.
20:12
Oh my god, I if if you have a lot of emails, you're going to have a lot of emails with customers. Oh man, it's
20:19
superhuman is amazing. It has a bunch of shortcuts. I love keyboard shortcuts and you just fly through emails. Um and it's
20:27
not like it's AI generated like it makes you write the email and you have AI help you but it's just a very focused uh
20:33
focused platform. And then lastly a sauna I use a sauna for internal facing so not customerf facing you know if I
20:39
want to keep track of some specifics around details of of what needs to be done. Um yeah that's the that's the
20:46
software stack. Um okay let's dive into the the agent side
Building Agents Stack
20:51
of things now. So, for building agents, the irony here is if you don't know how to build an agent, please don't worry. I
20:58
got you. We're going to use agents to build agents. Um, and so Cloud Code,
21:06
they have a new desktop app. It's awesome. OpenAI's Codeex, they have a new uh desktop app. It's awesome. And
21:14
you can actually use these to build the agents for your customer. Um, and as far
21:20
as what agents to use, you have a couple options. You're not going to sell Claude
21:25
Code to your customer. You could or Codeex. I mean, you could, but I highly
21:31
recommend using Hermes these days. I find it to be the most reliable. Um, it allows you to pick any model. The
21:38
reasoning here is tomorrow there's going to be a new model that comes out. It's going to be infinitely cheaper and it's
21:44
going to be Opus 4.7 level intelligence. And it's like you just want to have the flexibility to quickly switch whatever
21:51
the agent that you're running. Uh whatever model it's running, be able to switch that quickly and you don't want
21:56
to be married to a platform, married to a tool, married to um an infrastructure. So um Hermes I really like. Have you
22:05
played around with Hermes at all? I I think I saw some videos. Yeah. Yeah, I have. Um I haven't, you
22:12
know, quite made the shift yet. Uh but uh I've done an episode on it with my
22:18
friend Imran. So go check it out if people are interested in learning about how to set up Hermes. We called it
22:24
Hermes cuz we're fancy like that in the episode. And then we got, you know, the team at uh Hermes, you know, quickly
22:32
corrected us. They did. Oh wow. I I like Hermes. Yeah. Hermes is a little more fancy. If you if
22:38
you sell Hermes agents, you can charge 10K a month. Exactly. So, Open Claw's commoditized
22:45
already. You know, it's 5K a month. It's okay. So, you pick your harness here.
22:50
So, this is, you know, the agent that you'll sell and you need a place for that agent to live. You can use
22:56
something like Hostinger, you can use Orgo, you can use whatever. Um, I
23:02
obviously am biased. Um, but Orgo is really nice because in one workspace you
23:09
can have all your agents. You have your agent managing their agents. And I'll dive into all of this uh and getting set
23:14
up. And then lastly, you need the tools for the agents. Some things out of the
23:20
box that I install for every agent no matter what. Um, outside of just giving them a computer and the ability to use
23:26
it is Composeio. Have you heard of this company, Composio?
23:31
I have, but can you give a oneliner for folks who haven't heard of it heard of them?
23:36
This company allows you to this connector, they they allow you to have
23:41
one connector, one MCP essentially that connects to thousands of other apps,
23:47
whether it's Gmail, Slack, Notion, what have you. And with one connection, you can manage uh you can have access to all
23:55
the tools that you would need to send an email via Gmail or push something via GitHub or pull a message via Slack. It's
24:03
incredible. And it handles the tool the tool calling and the authentication which is huge because security is like
24:10
the biggest challenge of setting up these agents. Like by far the biggest
24:15
time sync is getting authentication set up for the customer because you have to h what's your username and password for
24:21
this and then if you email it it's like not secure. So then you use something like composio done. So it handles that
24:28
and then it handles security in that sense as well. Everything's managed through their platform and then it
24:34
manages handles the tool call. So, if you have Composeio set up with all the connectors, you can just take that one
24:39
connector, take it to any agent, and it has all the same connectors. Um, so I really like this company. I don't have
24:45
any affiliation, but I love their product. Um, really great. Next up is Agent Mail.
24:52
This one is I I give every agent uh an email. It adds a nice personal touch.
24:58
So, um you know, let's say you're you're an executive. I give you an agent. You
25:05
name it Mia and Mia needs Mia needs her own email.
25:11
Agent Mail allows you to give Mia an email so that she can send and receive emails and that's really fun because it
25:16
it turns into like truly like a personal uh assistant. And then lastly, Obsidian.
25:23
Um you have a video on Obsidian. It did really well. Obsidian super important
25:28
because at the end of the day, these agents need context. And the more context you can provide in a nicely wiki
25:35
styled structured format and markdown files for the agent, um it will really
25:42
just thrive in terms of understanding projects, people, things that you're doing, so on and so forth. So um this is
25:49
the stack. And as far as models, I guess final touch around models. Um, today, by
Model Picks: GPT 5.5, GLM 5.1, Kimmy, Opus 4.7
25:56
far the best model to use for something like a Hermes agent or an OpenClaw is GPT 5.5. Um, it's so efficient with the
26:06
tool calls. It doesn't eat through tokens like Opus 4.7 from Enthropic
26:11
does. Um, and and OpenAI is very generous around letting you use your
26:16
paid plan uh with with with any model like with any harness like like Hermes
26:22
or or Open Claw and then um and you just get a lot of usage out of it. So I
26:28
recommend 5.5. If you want to use open source models that are a little more affordable for lighter weight tasks, uh
26:34
GLM 5.1 from ZAI is in my experience like the best open source model to be
26:41
using. Kimmy comes in on a at a close second. Uh and these are both more affordable. Uh and then Opus 4.7.
26:49
Finally, if you have some long horizon coding task, um Opus 4.7 is really great
26:55
for that. And um you can actually have your agent uh connect to claude code and
27:02
be able to do these long coding tasks in cloud code and then bring that back to the agent. So um tidbit there. Uh I want
Nick’s Stack
27:10
I don't know if you can do this real quick but could you give a oneliner on because people are going to sorry let me
27:16
take a step back. People are going to look at this list and they're going to be like, "Oh my god, I don't know if I I should use codeex or if I should use cloud code, if I should use opencloud,
27:23
if I should use Hermes, should I use hosting or should I use orgo? Should I use this? Should I Can you go and just
27:30
quickly, you know, what's Nick's stack and like with a oneliner of why you use
27:36
that tool over the other tool?" Yeah. Codeex because it's more generous
27:43
and it's simplest and they have the best desktop app. Hermes because it doesn't break and it's self-evolving. Open Claw
27:50
is not as self-evolving. Uh Orgo because we give your agent a
27:55
computer so it can live in the computer. It can operate the computer. We're not just a headless VPS server in the cloud.
28:03
Um and I'll dive in on that. Composio, you need this. Everyone needs this.
28:08
Agent Mail, everyone needs this. Obsidian, everyone needs this. Um, and then
Why Obsidian Is the Second Brain Layer
28:14
that's a hot take by the way. Obsidian. Everyone needs this. Oh, yeah.
28:19
Yeah. Give just I mean explain why you should use Obsidian over say Notion.
28:27
So, here's my Obsidian vault. And Greg, I I've been I've been building this
28:33
vault since November of 2025. agents. I mean, a uh what do how do how do you say
28:41
um when something's outdated like super old 2025 that's forever ago, you know?
28:48
Um so I've been building this vault since 2025 November before open claw, before Hermes, and it has everything
28:55
about people, projects, everything. And I'm so crazy. I have a limitless
29:04
microphone. even that daily transcripts get pulled from that into here. Um, this
29:11
is genuinely a second brain. Like like people say Obsidian has a second brain and then okay, they they show it. Okay,
29:17
that's kind of cool. They use it for some research. No, no, no. This is a second brain. And when you have
29:22
something like this, it is quite literally you get to experience what
29:27
personal AGI might feel like uh in the next 3 to 6 months uh from now. I'm sure
29:34
everyone will experience it. But I feel like I'm getting to experience it sooner because I just have such well organized
29:40
uh markdown files. Um it's incredible. It's incredible. So it just gives your
29:46
agent context on what it needs to know given any given
29:52
tasks and it feels like it just never forgets and it understands you. Um, and
29:59
I think that's at the end of the day like we just want an agent that understands us and helps us with our
30:04
business and and just has perfect context over everything we do. So, enough said.
30:10
Yeah. And then 5.5 is the best. I would just use 5.5 to make it easy. Yeah. GPT
30:16
5.5. So, as far as that's the stack. Um, now
Live Walkthrough: Spinning Up a Cloud Computer in Orgo
30:22
in Orgo, we give the agent a a computer to live in. Greg, let me invite you to this. Um, all right. So, I just invited
30:30
you, Greg, into this workspace in Orgo, and we're just going to quickly spin up a computer here. And I'm going to spin
30:37
up a computer. I'm going to say, um, Greg's computer. I'm going to launch it.
30:44
And it launches pretty fast with really fast desktops. Um, and now that we're in
30:50
this workspace here in this computer, I can now install the agent inside of it.
30:55
actually. So the agent can live inside of here. So if it's open call, if it's Hermes agent in this case, it will live
31:01
inside of this environment. And um the key here is regarding to
31:07
getting set up, we have an orgo MCP that is what I use for for setting up agents.
31:15
And that little story I told earlier about going on a walk and be able to get work done on a walk. It's because my
31:22
agent is using a orgo MCP to connect to my customers agents that live on Orgo.
31:29
And so what ends up happening is Orgo is like this workspace where my agent and
31:35
other agents and myself can all collaborate on these computers where these agents live and get them set up
31:42
and configured that way. Um, so here I have a um I don't know if can you see my
31:49
Telegram chat? Yep. I had my agent last night actually I
31:54
kicked off a task. I told it to uh go ahead and build out some CLI and skills for uh for for Orgo. Um but I'll start a
32:03
new chat here and I can actually just tell the agent um I'll grab the computer
32:08
ID from orgo. So let's grab this computer ID and I can give this computer ID to the
32:15
agent and I'll say set this computer on orgo
32:22
up um computer ID quoted here.
32:28
Let's install Hermes agent into the VM.
32:34
So, the reason why I tell everyone not to get stressed out or scared about setting
32:41
up these agents is you really just need another agent to set it up. In my case, I'm using another Hermes agent to set up
32:48
a Hermes agent. In another case, you could here I'll spin up another computer
32:54
here. In another case, you can literally install something like cloud code into a
32:59
VM on orgo and you can actually just run cloud code
33:04
from the terminal here and tell cloud code in natural language, hey, let's set
33:10
up Hermes agent. Um, so just real quick, you go cloud code install command for
33:16
Linux. you find that real quick and you just run this in the terminal here and you
33:22
would literally install cloud code, run it from here and have it install Hermes into this VM. Um,
33:31
so the answer to all of our problems, Greg, is that more agents is the answer.
33:37
Uh, if you're confused on how to set something up, have your agent do it. Um,
33:44
and yeah, so I'm just going to install Cloud Code here. It's going to get that going and we'll be off to the races.
Cloud Computers vs. Mac Minis
33:53
This will be a dumb question, but why are we doing virtual computers versus
34:00
doing local computers? You know, buying Mac minis and and doing that whole thing. It's actually a very very good
34:08
question and the reason is we want the ability to work on our customers computers from
34:15
where we're at. And if you are using a Mac Mini,
34:21
I can't even imagine the nightmare of having to go in person and you know debug something that's like at a
34:26
hardware level or um something on the Mac Mini bricks or an update or or what have you. Orgo gives you cloud computers
34:34
to be able to manage these agents and um with that you can do so many more
34:39
amazing things both from a perspective of just scaling your business being able to access all these agents on one
34:45
platform via one connector and have your agent connect to all of them. Um that's
34:51
really like the biggest thing is just from a fulfillment perspective. Um it is just like the easiest way and then also
34:58
just a security perspective. These are isolated cloud computers and you can
35:03
delete them and under a second you can create a new one and with that there's a
35:08
lot more sandbox environments that you could just protect you and your customers from uh like a blast radius
35:15
that might otherwise be um more dangerous on a personal Mac mini. So
35:21
and so say I have 100 customers am I creating like how am I structuring that?
35:27
Am I creating like separate like projects with these computers in it? Like what is from a best practices
35:34
perspective in terms of security and just you know good UX h how should
35:40
people think about setting that up? Yeah. So in this case it would be exactly like you said like I if you were
35:47
if you were a customer um I would just make a workspace for your business and I
35:52
would say um you know this is let's do like idea browser right and we would
36:00
create this workspace and each each of your agents would live in this workspace and then I'd have other workspaces for
36:07
other customers and I'd be able to manage all of that you know on on orgo
36:12
Um, yeah, one platform. Cool. Yeah. I think what's also cool
36:19
about this just just how visual it is, like showing this to a customer and
36:24
being like, I know you think, you know, it's not secure. You might think it's not secure or you might think, you know,
36:30
but this is, you know, a visual sandbox environment, right? Like it just it feels like the cell like you just like
36:38
you know you talked about loom before but like showing looms of this I think is just going to light people up.
36:45
Yeah, exactly. And also like you can also out of the box on orgo like we have this
36:50
playground mode here. So like this is our this is just all the latest models from you know anthropic and and Kimmy
36:57
and and chatbt. And so as far as a demo, when you tell a customer like, "Oh yeah,
37:03
like we can we can have an agent like operate a computer and do do things for you and essentially you're describing
37:09
Hermes agent or you're describing OpenClaw." Even then they they might have a hard time like imagining like
37:15
what what does that look like? What does that feel like? And so when you just give it a computer, you're able to just
37:20
give it life. And you could tell them like you can say like hey look up what is idea browser um and search it on
37:28
Google and this actually becomes like a really good demo like for for you and
37:35
your customer like to be able to show look oh the agent is controlling a computer
37:41
and it's doing research and it's doing real work and you can just quickly show a demo in orgo. It's like super cool. Um
37:49
this is cool. Yeah. Like even me as like a co-founder of Idea Browser, I'm like looking at this and I'm like, "Yes."
37:56
Yeah. Yeah. It's awesome. And and we have some I'm I'm making Jordan some agents. I
38:02
don't know if he told you I'm making him some idea browser agents and he's uh he's using them. He's stress testing
38:08
them. Um but everyone needs agents. So it's um yeah, it's really cool. And then
38:16
as far as the telegram setup here, like you can see my agent is literally using or MCP to get this Greg's computer set
38:24
up right now and it's installing Hermes agent. Um you know part of this you know
38:29
these some of these things take time. It running a long process. It might take 5
38:34
10 minutes. So not to bore you by sharing that but um the concept of use
Building Agents and Structuring Workspaces for Customers
38:39
agents to set up other agents it's very real. Um, and I could also dive into
38:45
practices around that and how to make sure that your agent knows how to set up other agents. Um,
38:52
yeah. Does that sound good? Let's do that. That sounds great. So,
38:58
to have your agent have context of how to set up other agents, it actually needs I wish I would have added them
39:04
here. Um, a few MCPs go really far away. One of them is the perplexity MCP. Um,
39:13
with perplexity, you can give your cloud code or codecs
39:18
uh real real up-to-date knowledge on things like Hermes. And so the key here is like you always just want to have any
39:26
sort of setup process, initialization process grounded in real context of what
39:32
is the docs for setting up Hermes agents today and what how do I connect my Hermes agent to iMessage. If you have
39:38
something like perplexity, you give your agent the ability to see how to do that and be able to set it up perfectly. Um,
39:46
Exa AI is another great MCP tool for like real- time web search. Another big
39:52
one actually is Context 7. This one is awesome for getting up-to-date docs from
39:58
like GitHub from like Hermes agents GitHub so they they can see specifically
40:04
the docs of how to get set up. you just need some sort of context layer to lay to loop in the best practices and
40:10
up-to-date docs um for setting up these agents. And kind of like one final
40:16
recommendation would be the XMCP. So Twitter released their own MCP. And I
40:22
find so many amazing setups on Twitter for OpenClaw and Hermes agents that there's many times I just want to use
40:28
that context for setting up an agent uh for a given task. And you could actually use this, give this to your quad code or
40:35
give this to your uh codeex and have it um use this context to help help you set
40:40
things up. Um or you could use all of them too. So I mean is there any downside to using
40:46
all of them? Um no I I use all of them. Um and so
40:51
like maybe even in here when you look at my telegram u you might notice oh I
40:56
guess it's it's I didn't ask it to to pull in up-to-date context. Mine has skills already built in place to be able
41:03
to like set these things up just because I do it so much. But in general, like the more the marrier. Like context is
41:10
key. And um I like to like have sub agents spawn. I'll tell Codeex like hey
41:17
or Claude Code, hey spawn five sub agents. uh one sub agent for perplexity, one for exo, one for context 7, one for
41:24
fire crawl, one for um XMCP because I like to pull from different resources
41:31
and then those all come back to the main agent and um we get the best practices.
41:37
So that's how I do it. Cool.
41:42
Let's see. Okay, so this here is the I should have ran this in the terminal
41:48
below, but I'll go ahead. I'll I'll just run it in this terminal. And what we're going to do here is do that. Run claude.
41:58
You just like spin up multiple terminals. [laughter] Yeah,
42:03
you can. Oh, wait. Let me see. Um, okay. I think I just copy this here. So, okay.
42:13
Oh, command not found. Okay. I I'll debug this later. Um, this
42:20
is also why this is mainly why I use the Orgo MCP. I'm just like I let my agent
42:25
do all the work. Um, actually you could also come here into the playground and say install cloud code into this
42:32
computer. Um, and just have our agent do it because I I don't I'm I don't want to
42:37
debug what's going on in the terminal right now. So then just have this agent do it um and do it that way. But yeah,
42:45
once you have it set up from here, like I can now ask my Telegram agent. Um, so
42:52
I'll I'll stop this here. You can just imagine, you know, one that's done your
42:57
agent setup and I'll start a new chat and I'll ask like how many Orgo VMs do I
43:04
have in my workspaces? And my orgo claw
43:11
uh is able to actually manage all of my customers agents from you know just this
43:19
one agent and it can upgrade fix things on the fly you know and
43:26
all from one spot anywhere I'm at. Um, if I if I get an email from a customer
43:31
that uh something broke, we can just send off an agent uh send off a message
43:37
to Orgo Claw and have it go fix it. Um,
43:42
boom. You can see here 27 Orgo VMs across your workspaces, all 27 shows running
43:49
and then it dives in onto all the different customers and all their agents. Um so last point that I want to
Watchdogs, Observability, and Reliability
43:57
make around getting these things set up is the uh watchdocs. So the gateways are what
44:06
make these agents connect to a platform like Telegram or a platform like
44:12
WhatsApp. And sometimes these gateways crash. OpenClaw has a lot of gateway
44:18
issues in my experience. Hermes is a lot better. Um, and so a key here is you
44:23
want to make sure that you set up a um, watchdog. You could literally just tell
44:28
your agent set up a watchdog for whenever a gateway crashes that it auto restores it. Um, that's super important
44:37
just from, you know, reliability perspective. A second thing is you want to make sure that you have some layer of
44:45
observability or alerts. So, I have agents email me. If I set up your agent,
44:52
your Mia agent, and Mia has an email, Mi Mia's Mia emails me from her email when
45:01
her cron job breaks or her skill failed or something happened and I'm alerted
45:07
about it and I can go in and then debug it and fix it. um which is super
45:12
valuable cuz once again for your customer you don't want them to have to worry about like doing all this
45:18
themselves. So, um, make it as simple and easy as possible, handle everything
45:24
tiptoail. And yeah, I think I guess like the big
Closing Thoughts on the Solopreneur Era
45:30
takeaway here is it is hard to set up cloud code even like people are like
45:35
cloud code is is going to kill openclaw or cloud code's going to kill Hermes agent and in a general sense it's
45:41
getting better at doing a lot of these general things but to be able to go in and create a specific agent for a
45:47
specific industry and person and have it tailored to their workflow it's like
45:53
you're underestimating how much value that is and you can really create a lucrative business by yourself. Uh you
46:00
and your agent building other agents for other businesses. Um yeah, and I think it's an amazing time to be a soloreneur
46:07
for this. You can and you will. So Nick, thank you
46:12
for sharing the playbook for how to build a oneperson
46:18
agent business. uh sharing how to actually do it um in such a clear way. I
46:26
love chatting with you because you're you give the sauce, but you also explain it super clearly. Nick is a criminally
46:33
underfollowed account on on social media. Uh he you know, he's gaining some
46:39
some followers, but I think he can be I think he needs to be bigger. So, I'll include links for where to find Nick
46:45
uh in the show notes in the description. And uh Nick, I'll see you in a few weeks in San Francisco and let's have a let's
46:52
have some coffee and have a good time. Thank you, Greg. Always a pleasure. Thank you so much. And um I I hope to
47:00
Yeah, we're going to see you soon. We're going to get some coffee. We're going to we're going to do some sip time. [laughter]
47:05
We're going to do some IRL sipin time, which is my favorite. It's there's there's nothing like it, you know? Like
47:12
I actually have been trying to cut down on my like Zoom meetings and stuff like
47:17
that. It's just there's not there it is. There's nothing like being in person, sharing ideas, sipping and uh
47:27
and figuring out what what we can be building in in in a time like this
47:32
because there's so much. And sometimes the hardest part is figuring out the right idea, the right time, the right
47:38
playbook, the right steps, the right order. And uh this has been helpful, Nick, and and definitely got my c
47:45
creative juices flowing. So I'm sure others are very thankful as well. So thank you, Nick. And I will see you next
47:51
time. Thank you, Greg. Talk soon.

