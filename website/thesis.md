---
Date: 2024-03-17
---
	Words goal: 20 000 ish
	Citation format: 
	
# 1: Introduction

The theoretical complexity of a creative work has no upper bound. A project may consist of hundreds or thousands of elements and design decisions about how these elements relate with each other. Characters, environments, story beats and plot points, props, motifs of all kinds, symbols, and concepts, to name a few. They exist in context; they may reference other creative works or rely on real world knowledge for accuracy and immersion. 

The final work, then, is an arrangement of its elements in some cohesive and harmonious layout. The elements are laid out in overlapping and nested patterns. Guiding principles, intentions and considerations will inevitably develop, consciously or otherwise. There will be a logic and a rhythm to how the elements are presented and how they relate to one another. 

A lot of the work of making such a large piece of creative work goes into either implicitly or explicitly managing these patterns. There are design documents, outlines, timelines, mindmaps, scripts, storyboards and much more. These are ways of creating maps of certain discrete sections of the work, but they are often not explicitly connected to each other or to the actual substance of the work, the individual elements that it will be built out of. This means they have to be separately managed and updated, manually, creating room for error and for earlier design decisions to be forgotten.

Regardless of the severity of them, inconsistencies in creative works are almost universally undesired. At worst, they break immersion and cause the audience to leave the experience confused or even angry. 

Simply put, the managing of complexity in an expansive creative work is difficult. This thesis presents a concept and early prototype for an alternative tool for addressing this challenge. A qualitative user study of 2(?) working professionals in creative fields has been conducted for this thesis, which provides some of the scaffolding around its arguments. 

The research question is formulated as follows:

**How can dynamic semantic graphs help manage complexity in creative works? (Q1)**

Auxiliary questions:
How much time does a typical creative professional spend creating and updating project overviews? (Q2)

Chapter 2 covers the main theoretical body of the thesis. It begins from establishing a need for a first principles -approach, then covering the selected principles in a general order of primacy. These principles also guided the development of the tool. They primarily come from three sources:
* Cognitive science as it pertains to memory and creativity
* Technological topics such as the Semantic Web
* Observed shortcomings in existing solutions and workflows. 

Chapter 3 will ground the theoretical in the concrete. It will go over the conducted user study, providing more real world examples of the research problem and the existing solutions and coping strategies to it. Participants were shown the software prototype, and they share their impressions of it. [?]

Chapter 4 will introduce the software prototype, which has been named Karta. This chapter will cover its development process in brief, and then move on to using it to outline and design a short story. While the prototype is functional, it is missing a lot of the features which it would need to properly fulfill its purpose. As such, a formal usability study was not conducted. 

The focus in this thesis is to establish a set of first principles. These principles could form the basis of a promising new area for future research and application, and Karta is an initial example of a tool belonging in this category. 


# 2 - Theory / Problem Space
## 2.1 - First Principles Thinking
	First it is useful to go over the justification for this lengthy theory chapter. 

## 2.2 - Atomics
Everything in the universe is atomic. Not in the sense of having atoms, but in the sense of being divisible into ever smaller units. Larger structures always emergent from the behavior of these units, which are discrete[?]. A somewhat arbitrary distinction can be made when it comes to structures created by living organisms. 

Emergence is the phenomenon of complexity arising from the concurrent interactions of simple units. The snowflake is a timeless example of this. The simple behavior of cold water particles interacting can form incredibly intricate structural patterns. The universe consists of just a few fundamental forces and particles, yet it still gives rise to massively detailed and complex processes, such as life itself. 

Creative works, much like the real world, have a structure. They have patterns. This is self-evident[?]. The only difference to natural emergence is that there may be conscious intention at play in the arrangement of patterns in a creative work. Though some could argue that even this is just emergence with extra steps [?]. 

Any work can be structurally divided into units, pieces which themselves are practically indivisible. A painting, for instance, can be said to consist of brush strokes, visually identifiable shapes, or pixels if in digital form. A piece of music, at its barest, is made up of individual waves of sound. A novel can be divided into scenes and arcs. There are a myriad of ways in which a thing may be divided and categorised. The reverse of division is composition, where there is likewise a combinatorially explosive amount of possible arrangements. The more "mass" there is, the more ways there is for it to be divided, and the more pieces there are, the more there are ways to assemble them. 

In computer graphics, the most indivisible units are referred to as primitives[?]. A vertex is a primitive; it represents a single point in space. Points can be linked together to form edges and polygons, also types of primitives. With enough of these primitives, you can represent any three dimensional surface or object to an approximate level of detail. Rarely are individual polygons or vertices referenced; usually they are grouped into vertex groups and those groups are referred to more abstractly. 


## 2.3 - Cohesion and Consistency
Why is cohesion important in creative work? Why would creators find it desirable?

The structure of a creative work does not only exist in the design documents and final artifacts, but it also appears in the minds of those who experience it. Each reader of a comic and player of a game forms their own internal mental models of the work. Where there are gaps, they might fill them. Where there are inconsistencies, they might discover them. 

Most, if not all creative works, regardless of scale, have a purpose. If it were not so, they would not exist[?]. The work has an intended effect or message it was designed to convey. The structure of the artifacts that the consumers experience guides the formation of their mental models, which constitute the final experience. 

There is reason to believe that certain forms of order are inherently pleasurable, interesting or otherwise attention grabbing for humans. Cohesion adds a tinge of predictability to the experience, setting the rules for what can and can't happen. The consumer develops expectations, and unintentional or haphazard subversion of these expectations is experienced as jarring. It breaks immersion. Even in an individual image, if the choice of color (or any other pattern) abruptly changes in some part without any apparent reason, it would break the harmony of the whole.[?] 

Perhaps there is no need to even look further than the attempts throughout millennia to declare rules and ideal patterns for all mediums of art and creation. A myriad of principles have been proposed, usually in relation to specific mediums. Most can be abstracted so as to be applicable to multiple mediums. Examples include composition, contrast, balance, harmony, resonance, tension, and resolution. By no means an exhaustive list.

It is clear that establishing intentional complex patterns is a goal for almost all creative works. There are no true exceptions to this [?]. Even works that strive to be as incoherent as possible still have the overarching goal of being incoherent, making them paradoxically coherent again. 

In sequential storytelling mediums, events unfold or are presented to the audience so as to guide them to develop a particular mental model of a story. Information about the story can be given, implied, or omitted. This is important, because it is not common for a story to unfold clearly, in full view, giving the audience all of the information about the plot and characters chronologically. Often there are details that are left out or brushed aside to be made relevant later. The experience of the audience is carefully directed.

An example of such direction is foreshadowing. Foreshadowing is a writing technique where an important event or detail in a story is subtly hinted at long before its full importance is made clear. This technique is often employed early and throughout a story in anticipation of a plot twist or a grand reveal. When the reveal happens, the meanings of the previous hints and details are suddenly obvious, and the audience can feel like their entire perspective on the story has shifted.

The effectiveness and prevalence of foreshadowing in storytelling speaks to the layers of meaning that creative works can employ. There are surface-level meanings which are quickly grasped by the audience and create the initial impressions. New information can later recontextualise these early meanings and add another layer beneath them. For example, a character may have a strong personality trait which creates a distinct first impression of them. Later on a detail in their personal history could be revealed that elaborates on the origins of their personality trait, sometimes entirely changing the audience's general opinion on them. Later still, information about the entire plot or the story-world[?] can further change the outlook on the character and their role in the story. 

Multiple layers of meaning reward those who engage with a work deeply and multiple times. These layers in fact invite the audience to engage more deeply and to do it together as well, forming communities. There is ample reward for creators who manage complexity such that there is both space for those who are new and those who want to delve deeper. 

## 2.4 - Overviews 
How, then, are the desired patterns to be established or existing patterns improved? Most commonly by creating overviews of them. 

![[Mind In Motion#^41d221]]

Creating a cohesive whole requires moving through all the distances, looking at the work both from afar and from up close. 

On some smaller scale works, such as individual paintings, this is fairly straightforward. The painter can see their whole painting at once, paying attention to how areas of light and dark interact and how the patterns create the impression of the complete image. The level of skill in observation of patterns and in manipulation of paint, of course, present a limit in how far the patterns can be pushed in the desired direction. But in the creation of a single image, the painter is granted access to any scale at any moment. They can look closer, to work on the details, or they can move back to see the entire image. They pick and focus on a pattern that they are interested in modifying. 

The advantage of singular images in particular is that the elements and the final work are inextricably linked. Added details in some corner of the work will be immediately visible in the overview. This is a luxury that is lost in larger projects which don't have an obvious singular visual representation. 

Consider for example a short film. This hypothetical short film runs for 8 minutes and has 11 scenes in it, totaling 36 different shots. At 24 frames per second, the entire 

In a larger project, overviews have to be created separately. They are often disconnected from the substance, the actual frames that constitute the final animation. 

Generally, production proceeds from the top, down. There are established production processes in the film industry and many of them are mirrored in medium-scale productions such as comic books and independently published games. 

## 2.4 - Externalization

	Where does this section fit, really? Could potentially be merged with Overviews. Though these are somewhat different angles on a similar topic. 

Externalization of knowledge improves both knowledge retention and communication[?]. Externalization often means visualization, because a visual layout of information does not impose a tempo or direction for the consumption of that information. A written treatment imposes an order and a direction, and an audio recording also imposes a tempo. Even if you change the speed of a recording, all sections are sped up or slowed down at the same rate. What exists in the mind of the designer and the storyteller is a structure, a mental model, of the creative project. The creator in question can then navigate this structure in their own minds[?]. 

![[Mind In Motion#^8ef229]]

Externalization is the creation of a representation of this structure. The representation is then given to other people to navigate and explore, just like the original creator can do with the mental model. It stands to reason that the more closely this external representation matches the creator's internal model, the more information can be successfully transferred to others. 

Another benefit of an external representation is that it does not covertly change in subtle but potentially important ways in the mind. Memories change when they are accessed in the mind[?]. This happens both consciously and unconsciously. While remembering something, you might come up with a new relationship between two previously independent ideas. Or not recalling some connection while remembering others weakens that memory in relation to those others, changing the overall structure. The brain does not seem to distinguish between access and modification[?]. It is not possible to simply access a memory without risking contamination. 

Physical, external structures can, however, make this distinction. 

These benefits apply both to teams as well as work done by individuals. Indeed, due to the nature of the mind, how memories change over time, and how cognition and perception can be modulated by emotions and other states of mind, it can be said that the individual is also working in a team. Communicating clearly to yourself across time is just as important as to others through space. 

## 2.5 - Conceptual Space
	This section begins the process of approaching a solution.

Finding the most appropriate form for representing thought requires approximating how these structures exist in the mind. 

![[Mind In Motion#^34093b]]

![[Mind In Motion#^b04719]]

![[Mind In Motion#^baa04c]]

Mindmaps are a conceptual tool ubiquitous almost everywhere, from primary schools to science labs. They are extremely flexible. 





# 3 - Practice
## 3.1 - The User Study

## WIP thoughts


# 4 - Karta

For this thesis, a prototype has been produced, which implements the fundamentals from the theory chapter and establishes the plausibility of putting the theory into practice. Due to the scope of such a software project, its full production could not possibly fit into a single master's thesis. The emphasis is instead on presenting it as one possible implementation of the theoretical principles and acknowledging that more or better ones could exist. In other words, this section is about grounding the 

## 4.0 - Design Goals and Requirements

In this section we will define design goals and requirements for a software tool that could work in accordance to the principles laid out in the theory chapter.



There are hundreds of different software and file formats for authoring assets and complete creative works. There is the concept of the pipeline, which is how all the different tools work together and how they are used both sequentially and in parallel to author assets and complete scenes. Commonly studios and individuals develop their own workflows, automating them either implicitly through routine and procedural memory, or explicitly through scripts and other programs. No tool is monolithic these days, at least not reliably. Even a Swiss army knife such as Blender has weaknesses that are supplemented with other tools. 

## 4.1 The Semantic Web / Graph
	Is this a theoretical chapter? Does this belong in 2? Perhaps one of the last sections in 2?
n
The [[Semantic Web]] was a new hypothetical iteration of the world wide web coined and chiefly championed by [[Tim Berners-Lee]] . The core idea of it has many parallels to the topic of this thesis, albeit on a much larger scale. The internet was noted to have become an amorphous mass of mostly unstructured data, individual pieces of which were understandable to humans but which computers couldn't reason about. The proposed solution was to formalise the structure of the web into discrete entities and embed semantic data describing their relationships. So it was all about an addition of metadata. 

Metadata is, plainly, data that describes other data. All files have properties and metadata that can be stored of them. File size is a great example. It is simply the information about how much information there is in a given file. If a file were to be categorised in some way, that would be done via metadata. The relationship of a file to another is also metadata. [? What about if a file directly references another inside itself? Is it still "outside" metadata?]

Large creative projects are in similar situation to the Web. The data which constitutes the totality of a project is unstructured by default, which makes it hard to reason about the meaning of the data. What is needed is an outside indexing structure, a scaffolding if you will, that allows for the creation and modification of a semantic structure. 

Many composition and authoring software have incorporated linking features, alleviating the issue of disjointedness somewhat. A software can, for example, know which other files are imported into a scene and detect changes in them to update the scene. These links are information about inter-file relationships. They are the kind of semantic data we are talking about. They are however contained within the file that composes the others. This semantic data is not easily queriable or accessible outside of it; the constituent files aren't aware of their presence in the larger composition. 

We established [?] in the earlier theoretical chapters that fluid traversal of the full semantic structure of a project would be helpful to anyone working on it. 

# 5 - Discussion
# 6 - Conclusion