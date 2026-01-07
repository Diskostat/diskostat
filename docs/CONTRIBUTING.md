# Contributing to diskostat

## Info

- [Miro board](https://miro.com/app/board/uXjVNSZVn78=/)
- [Kanban board](https://gitlab.fi.muni.cz/xhercko/diskostat/-/boards) (in move)

## Development process

Here, the process of how we work is described. It's described in the way
of one portion of work to implement the product.

### Ticket

One portion of work to implement the product. In this project there
are two kinds of tickets:

#### Bug

severity = visibility (how often the user encounters it) *(times) impact (crash? cannot use? just UI thing?)
<!-- resource: https://sqa.stackexchange.com/a/7548 -->


#### Issue

- one task/idea, unit of work

### The flow

Next we have established following flow for tickets.
Each step in the flow is represented by.

#### 1. Open

No label is needed here. It's just in project's backlog. Here goes
everything first: bug, simple idea, big idea, ...

Tickets here are ready for refinement in a team meeting.

Agree on: how it will look like when implemented, happy flow, error
handling, requirements, non funcitonal requirements, design, etc.

Here the ticket is:
- taken down as an idea.

#### 2. Ready for taking

After refinement the ticket is ready to be developed. Anyone could pick it
and start coding.

Here the ticket is:
- well described,
- agreed by whole team on the ticket's outcome.

#### 3. In progress

That's quite clear.

Here the ticket is:
- in the making,
- occupied by a developer.

#### 4. Ready for review

Note that the ticket can be taken back to In progress if further
progress is needed. It also does not have to be put back.

Here the ticket is:
- in review,
- waiting to be reviewed.

#### 5. Closed

Here the ticket is:
- implemented,
- merged to main.


### Working with GitHub

- Create branch named feature/your-feature-name for each task.
- Create branch named fix/your-bug-name for each task.
- Use conventional commits.
- Developer creates MR to main. After approval, the developer merges the request.

### Releasing

#### Homebrew custom tap

1. Create release with tag of repo with sources.
2. Copy url to `.tar.gz` generated.
3. Create formula with `brew create --rust --tap <github's repo where's the tap> <url to the tar from previous step>`
4. Copy the formula file generated and distribute on the tap's github.
