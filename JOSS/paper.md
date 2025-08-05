---
title: 'STEPS: Serially Transferred Evolving Population Simulator'
tags:
- Rust
- experimental evolution
- bacterial population genetics
- microbial evolution
authors:
- name: Devin M. Lake
  orcid: 0000-0002-9974-2367
  corresponding: true
  equal-contrib: true
  affiliation: "1, 2"
- name: Zachary W. D. Matson
  orcid: 0000-0002-8445-3618
  equal-contrib: true
  affiliation: "3"
- name: Minako Izutsu
  orcid: 0000-0002-3291-3256
  affiliation: "2, 4, 5"
- name: Richard E. Lenski
  orcid: 0000-0002-1064-8375
  affiliation: "1, 2, 4"
affiliations:
- name: Department of Integrative Biology, Michigan State University, USA
  index: 1
- name: Ecology, Evolution, and Behavior Program, Michigan State University, USA
  index: 2
- name: Department of Computer Science and Engineering, Michigan State University, USA
  index: 3
- name: Department of Microbiology, Genetics, and Immunology, Michigan State University, USA
  index: 4
- name: DeSC Healthcare, Shibuya, Tokyo, Japan 
  index: 5
date: 13 August 2025
bibliography: paper.bib
---
# Summary

Bacteria and other microbes are widely used in the field of experimental evolution [@Lenski:2017; @McDonald:2019; @Ascensao:2025]. In many of these experiments, the populations are propagated by periodic serial dilutions into fresh media, with the microbes growing to a maximum population size between successive transfers. Here we present the Serially Transferred Evolving Population Simulator (STEPS) software, which simulates the dynamics of asexual populations as they grow and evolve in these experiments. The STEPS software allows new mutations to occur while a population grows, including beneficial, neutral, and deleterious mutations. The resulting lineages then grow at different rates according to the fitness effects of their accumulated mutations. After the total population reaches the maximum size, the dilution factor determines the proportion of individuals that are randomly chosen to start the next growth cycle. The differential growth rates of the lineages reflect the process of natural selection, while the periodic bottlenecks impose random genetic drift.

The underlying mechanics of the STEPS program are designed to simulate the biological processes that occur in actual microbial evolution experiments, while taking advantage of computational methods that allow the simulations to proceed quickly and efficiently. For example, the timesteps used in the simulations coincide roughly with the population’s doubling time, providing meaningful timepoints for introducing mutations. The mutational categories, the effects of mutations on relative growth rates, and the serial transfer process have been implemented such that the dynamics of the simulated populations reflect the same interplay of mutation, selection, and drift that occurs in biological experiments.

# Statement of need

The Long-Term Evolution Experiment (LTEE) with *E. coli* was started in 1988 [@Lenski:1991; @Lenski:2023] and it continues to this day [@Barrick:2023]. The LTEE has been used to study many aspects of evolutionary dynamics [@Blount:2008; @Wiser:2013; @Tenaillon:2016; @Couce:2024], and it has inspired many other experiments that use similar approaches [@Johnson:2021; @Izutsu:2024; @Stroud:2025]. Various publications based on these projects include simulations to analyze and interpret the experimental results. The authors of these papers typically write new simulation software that is custom-built to a specific experimental system or result, and even publications based on the same system may use different simulation methods [e.g., @Good:2017; @Wiser:2013]. We anticipate that future studies will be able to use STEPS to perform relevant quantitative analyses or interpret qualitative results. Researchers may also use STEPS to design new experiments or as a framework for building customized simulations. We expect that employing the STEPS program in these various ways will improve consistency and efficiency across studies.

# Key features

To enable STEPS to be accessible to as many researchers as possible, we have produced two different ways to run the simulations. The first approach requires using a command-line interface to run STEPS, and it is intended for advanced applications, such as generating data for large numbers of replicate populations for statistical analysis and comparison to experimental results. All features and options built into the STEPS software are available in this version. The second approach uses a web-based graphical user interface that provides immediate visualization of the results, helping researchers develop their intuition about the dynamics of evolving populations under various scenarios. This version of STEPS can also be used in educational settings, allowing students without computational backgrounds to obtain results quickly and easily. Some features of the command-line version are not available in the web-based version. However, both versions use the same underlying code while the simulations are running.

We show figures below to illustrate a few of the dynamical outputs that are available with the STEPS package. \autoref{fig:1} shows the trajectories of (A) average fitness and (B) average accumulated mutations for 12 simulated populations over the course of 2,000 generations. The essential biological parameter values are listed in the figure legend; they correspond to the approximate values for the LTEE. Both trajectories exhibit steplike dynamics that are typical of large microbial populations in which *de novo* mutations drive adaptation [@Lenski:1994], and which inspired the name of the STEPS program. 

![Trajectories for (A) average fitness and (B) average accumulated mutations in 12 simulated populations. The data were produced using the command-line version of STEPS with the following key parameter values: number of transfers = 300, maximum population size = 5e8, dilution factor = 100, rate of beneficial mutations = 1.7e-6, average beneficial effect size = 0.012 (drawn from an exponential distribution), rate of neutral mutations = 0.001, rate of deleterious mutations = 0.001 (drawn from a uniform distribution), strength of epistasis = 6.0, and initial seed = 606. All other parameters and settings are default values, except that an optional metagenomic dataset was saved (mutation-summary-output) and used to produce the next figure.\label{fig:1}](Figure 1 STEPS JOSS.png)

\autoref{fig:2} plots the metagenomic data for one of the 12 populations from the optional file that recorded the trajectories of every mutation that reached a threshold frequency. One can see a clean selective sweep in the first few hundred generations, followed by more complex dynamics as beneficial mutations (alone and in combination) compete with one another. This competition leads to the phenomenon of clonal interference, whereby most beneficial mutations are outcompeted by lineages that have acquired mutations with even larger beneficial effects [@Gerrish:1998; @Levy:2015].

![Trajectories for the frequencies of individual mutations in one of the populations in \autoref{fig:1} (brown trajectory, third population from left to show a conspicuous increase in fitness.) Only those mutations that reached a frequency of at least 0.01% were saved in the output file. See Figure 1 for parameter values and other details.\label{fig:2}](Figure 2B.png)

To facilitate the use of STEPS by users with diverse interests and skillsets in both research and educational settings, we have produced a User Manual [@Lake:2025] that explains: the scientific context and purpose of the STEPS software (Chapter 1); the use of the web-based version, including numerous exercises with figures (Chapter 2); the setup and full set of options available in the command-line version (Chapter 3); and the mechanics of the simulations that underpin both the command-line and web-based versions (Chapter 4). \autoref{fig:3} is a screenshot of the web interface along with results generated in a matter of seconds using the default settings.

![Screenshot of the interface and results using the web-based version of STEPS. Default parameter values were used, except that the random seed was set to 606. These values are the same as used for \autoref{fig:1}, except that the default values for the neutral and deleterious mutation rates are set to 0 to allow faster runtimes. Exercises presented in the User Manual [@Lake:2025] explore the effects of adding these types of mutations and varying population sizes, run durations, and various other parameters and options.\label{fig:3}](Default_Screenshot.png)

# Acknowledgments

Development of the STEPS program has been supported by the U.S. National Science Foundation (DEB-1951307), a USDA Hatch Grant (MICL12143), and the John A. Hannah professorial endowment at Michigan State University.  We thank Ben Good for valuable discussions during the design of STEPS, and Kyle Card and Nkrumah Grant for testing earlier versions of STEPS. The LTEE inspired us to create STEPS, and we thank everyone who has worked on that experiment over the several decades that it has been running.


# References
